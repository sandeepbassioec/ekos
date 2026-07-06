//! gRPC service implementation — maps proto RPCs to KoStore syscalls.

use tonic::{Request, Response, Status};

use ekos_kernel::KoStore;

use crate::mapping;
use crate::proto::ekos::v1::{self as pb, kernel_service_server::KernelService};

pub struct EkosKernelService {
    store: KoStore,
}

impl EkosKernelService {
    pub fn new(store: KoStore) -> Self {
        EkosKernelService { store }
    }
}

fn kernel_err(e: ekos_kernel::KernelError) -> Status {
    use ekos_kernel::KernelError::*;
    match e {
        NotFound(msg)       => Status::not_found(msg),
        Deleted(msg)        => Status::failed_precondition(format!("deleted: {msg}")),
        InvalidArgument(msg)=> Status::invalid_argument(msg),
        Store(msg)          => Status::internal(format!("store: {msg}")),
        Serialize(msg)      => Status::internal(format!("serialize: {msg}")),
    }
}

#[tonic::async_trait]
impl KernelService for EkosKernelService {
    // ── Syscall 1 ──────────────────────────────────────────────────────────

    async fn ko_put(
        &self,
        req: Request<pb::KoPutRequest>,
    ) -> Result<Response<pb::KoPutResponse>, Status> {
        let inner = req.into_inner();

        if inner.namespace.is_empty() {
            return Err(Status::invalid_argument("namespace is required"));
        }
        if inner.kind.is_empty() {
            return Err(Status::invalid_argument("kind is required"));
        }

        let ko = mapping::proto_to_ko(&inner)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // If an id was supplied and a previous version exists, let KoStore
        // handle the version increment.  Just pass through.
        if !inner.id.is_empty() {
            // ko_put in the store handles existing-id case
        }

        let saved = self.store.ko_put(ko).map_err(kernel_err)?;
        Ok(Response::new(pb::KoPutResponse {
            ko: Some(mapping::ko_to_proto(saved)),
        }))
    }

    // ── Syscall 2 ──────────────────────────────────────────────────────────

    async fn ko_get(
        &self,
        req: Request<pb::KoGetRequest>,
    ) -> Result<Response<pb::KoGetResponse>, Status> {
        let r = req.into_inner();
        let ko = self.store.ko_get(&r.namespace, &r.id).map_err(kernel_err)?;
        Ok(Response::new(pb::KoGetResponse {
            ko: Some(mapping::ko_to_proto(ko)),
        }))
    }

    // ── Syscall 3 ──────────────────────────────────────────────────────────

    async fn ko_delete(
        &self,
        req: Request<pb::KoDeleteRequest>,
    ) -> Result<Response<pb::KoDeleteResponse>, Status> {
        let r = req.into_inner();
        self.store.ko_delete(&r.namespace, &r.id).map_err(kernel_err)?;
        Ok(Response::new(pb::KoDeleteResponse { ok: true }))
    }

    // ── Syscall 4 ──────────────────────────────────────────────────────────

    async fn ko_scan(
        &self,
        req: Request<pb::KoScanRequest>,
    ) -> Result<Response<pb::KoScanResponse>, Status> {
        let r = req.into_inner();
        let kind = if r.kind.is_empty() { None } else { Some(r.kind.as_str()) };
        let limit = if r.limit == 0 { None } else { Some(r.limit as usize) };

        let kos = self.store.ko_scan(&r.namespace, kind, limit).map_err(kernel_err)?;
        let total = kos.len() as u32;
        Ok(Response::new(pb::KoScanResponse {
            kos: kos.into_iter().map(mapping::ko_to_proto).collect(),
            total,
        }))
    }

    // ── Syscall 5 ──────────────────────────────────────────────────────────

    async fn ko_history(
        &self,
        req: Request<pb::KoHistoryRequest>,
    ) -> Result<Response<pb::KoHistoryResponse>, Status> {
        let r = req.into_inner();
        let versions = self.store.ko_history(&r.namespace, &r.id).map_err(kernel_err)?;
        Ok(Response::new(pb::KoHistoryResponse {
            versions: versions.into_iter().map(mapping::ko_to_proto).collect(),
        }))
    }

    // ── Syscall 6 ──────────────────────────────────────────────────────────

    async fn ko_query_tag(
        &self,
        req: Request<pb::KoQueryTagRequest>,
    ) -> Result<Response<pb::KoQueryTagResponse>, Status> {
        let r = req.into_inner();
        let limit = if r.limit == 0 { None } else { Some(r.limit as usize) };
        let kos = self.store.ko_query_tag(&r.namespace, &r.tag, limit).map_err(kernel_err)?;
        let total = kos.len() as u32;
        Ok(Response::new(pb::KoQueryTagResponse {
            kos: kos.into_iter().map(mapping::ko_to_proto).collect(),
            total,
        }))
    }
}
