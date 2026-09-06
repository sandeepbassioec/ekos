//! Real TPM 2.0 sealing via tss-esapi. PCR policy: PCR0 (firmware) + PCR7 (secure boot).
//! Compiled only when target_os = "linux" AND feature = "tpm".

#[cfg(all(target_os = "linux", feature = "tpm"))]
mod inner {
    use crate::error::{TpmError, TpmResult};
    use std::path::Path;
    use tss_esapi::{
        Context,
        attributes::{ObjectAttributesBuilder, SessionAttributesBuilder},
        constants::SessionType,
        handles::KeyHandle,
        interface_types::{
            algorithm::{HashingAlgorithm, PublicAlgorithm, SymmetricMode},
            key_bits::AesKeyBits,
            resource_handles::Hierarchy,
            session_handles::AuthSession,
        },
        structures::{
            Digest, DigestValues, PcrSelectionListBuilder, PublicBuilder,
            SymmetricCipherParameters, SymmetricDefinitionObject,
        },
        tcti_ldr::TctiNameConf,
    };

    const SEALED_MAGIC: &[u8] = b"VVTPM1";

    fn open_ctx() -> TpmResult<Context> {
        let tcti = TctiNameConf::from_environment_variable()
            .or_else(|_| "device:/dev/tpm0".parse())
            .map_err(|e| TpmError::Device(e.to_string()))?;
        Context::new(tcti).map_err(|e| TpmError::Device(e.to_string()))
    }

    fn pcr_policy_session(ctx: &mut Context) -> TpmResult<AuthSession> {
        let trial_key = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Trial,
                tss_esapi::structures::SymmetricDefinition::Aes {
                    key_bits: AesKeyBits::Aes128,
                    mode: SymmetricMode::Cfb,
                },
                HashingAlgorithm::Sha256,
            )
            .map_err(|e| TpmError::Device(e.to_string()))?
            .unwrap();

        let pcr_sel = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[
                    tss_esapi::interface_types::reserved_handles::Pcr::Pcr0,
                    tss_esapi::interface_types::reserved_handles::Pcr::Pcr7,
                ],
            )
            .build()
            .map_err(|e| TpmError::Device(e.to_string()))?;

        ctx.policy_pcr(trial_key, Digest::default(), pcr_sel)
            .map_err(|e| TpmError::Seal(e.to_string()))?;

        Ok(trial_key)
    }

    pub fn is_tpm_present() -> bool {
        open_ctx().is_ok()
    }

    pub fn seal(data: &[u8], output_path: &Path) -> TpmResult<()> {
        let mut ctx = open_ctx()?;

        let policy_session = pcr_policy_session(&mut ctx)?;
        let policy_digest = ctx
            .policy_get_digest(policy_session)
            .map_err(|e| TpmError::Seal(e.to_string()))?;

        let sealed_obj_pub = PublicBuilder::new()
            .with_public_algorithm(PublicAlgorithm::KeyedHash)
            .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
            .with_object_attributes(
                ObjectAttributesBuilder::new()
                    .with_user_with_auth(true)
                    .build()
                    .map_err(|e| TpmError::Seal(e.to_string()))?,
            )
            .with_auth_policy(policy_digest)
            .with_keyed_hash_parameters(tss_esapi::structures::KeyedHashParameters::new(
                tss_esapi::structures::KeyedHashScheme::Null,
            ))
            .with_keyed_hash_unique_identifier(tss_esapi::structures::KeyedHashDigest::default())
            .build()
            .map_err(|e| TpmError::Seal(e.to_string()))?;

        let sensitive = tss_esapi::structures::SensitiveData::try_from(data.to_vec())
            .map_err(|e| TpmError::Seal(e.to_string()))?;

        let (private, public) = ctx
            .execute_with_sessions(
                (
                    Some(tss_esapi::interface_types::session_handles::AuthSession::Password),
                    None,
                    None,
                ),
                |ctx| {
                    ctx.create(
                        KeyHandle::from(tss_esapi::handles::PersistentHandle::from(
                            tss_esapi::constants::handles::Persistent::Persistent0x81000001,
                        )),
                        sealed_obj_pub,
                        None,
                        Some(sensitive),
                        None,
                        None,
                    )
                },
            )
            .map_err(|e| TpmError::Seal(e.to_string()))?;

        let pub_bytes = public
            .marshal()
            .map_err(|e| TpmError::Seal(e.to_string()))?;
        let priv_bytes: Vec<u8> = private.into();

        let mut blob = Vec::new();
        blob.extend_from_slice(SEALED_MAGIC);
        let pub_len = pub_bytes.len() as u32;
        blob.extend_from_slice(&pub_len.to_le_bytes());
        blob.extend_from_slice(&pub_bytes);
        blob.extend_from_slice(&priv_bytes);

        std::fs::write(output_path, &blob)?;
        Ok(())
    }

    pub fn unseal(input_path: &Path) -> TpmResult<Vec<u8>> {
        let blob = std::fs::read(input_path)?;
        if !blob.starts_with(SEALED_MAGIC) {
            return Err(TpmError::BadBlob);
        }
        let blob = &blob[SEALED_MAGIC.len()..];
        if blob.len() < 4 {
            return Err(TpmError::BadBlob);
        }
        let pub_len = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
        if blob.len() < 4 + pub_len {
            return Err(TpmError::BadBlob);
        }
        let pub_bytes = &blob[4..4 + pub_len];
        let priv_bytes = &blob[4 + pub_len..];

        let public =
            tss_esapi::structures::Public::unmarshal(pub_bytes).map_err(|_| TpmError::BadBlob)?;
        let private = tss_esapi::structures::Private::try_from(priv_bytes.to_vec())
            .map_err(|_| TpmError::BadBlob)?;

        let mut ctx = open_ctx()?;

        let policy_session = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Policy,
                tss_esapi::structures::SymmetricDefinition::Aes {
                    key_bits: AesKeyBits::Aes128,
                    mode: SymmetricMode::Cfb,
                },
                HashingAlgorithm::Sha256,
            )
            .map_err(|e| TpmError::Device(e.to_string()))?
            .unwrap();

        let pcr_sel = PcrSelectionListBuilder::new()
            .with_selection(
                HashingAlgorithm::Sha256,
                &[
                    tss_esapi::interface_types::reserved_handles::Pcr::Pcr0,
                    tss_esapi::interface_types::reserved_handles::Pcr::Pcr7,
                ],
            )
            .build()
            .map_err(|e| TpmError::Device(e.to_string()))?;

        ctx.policy_pcr(policy_session, Digest::default(), pcr_sel)
            .map_err(|e| TpmError::Unseal(e.to_string()))?;

        let parent_key = KeyHandle::from(tss_esapi::handles::PersistentHandle::from(
            tss_esapi::constants::handles::Persistent::Persistent0x81000001,
        ));

        let obj_handle = ctx
            .execute_with_sessions(
                (
                    Some(tss_esapi::interface_types::session_handles::AuthSession::Password),
                    None,
                    None,
                ),
                |ctx| ctx.load(parent_key, private, public),
            )
            .map_err(|e| TpmError::Unseal(e.to_string()))?;

        let sensitive = ctx
            .execute_with_sessions((Some(policy_session), None, None), |ctx| {
                ctx.unseal(obj_handle.into())
            })
            .map_err(|e| TpmError::Unseal(e.to_string()))?;

        Ok(sensitive.value().to_vec())
    }
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
pub use inner::{is_tpm_present, seal, unseal};
