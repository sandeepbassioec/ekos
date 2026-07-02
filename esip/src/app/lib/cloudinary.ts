/**
 * Cloudinary Upload Utility
 */

const CLOUD_NAME = 'dawxoue2h'; // Your cloud name
const UPLOAD_PRESET = 'esip_documents'; // Unsigned upload preset

export interface CloudinaryUploadResult {
  url: string;
  publicId: string;
  format: string;
  bytes: number;
  secureUrl: string;
}

/**
 * Upload file to Cloudinary (browser-based unsigned upload)
 */
export async function uploadToCloudinary(
  file: File,
  folder: string = 'documents'
): Promise<CloudinaryUploadResult> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('upload_preset', UPLOAD_PRESET);
  formData.append('folder', folder);

  try {
    console.log('Uploading to Cloudinary...', { file: file.name, folder });

    const response = await fetch(
      `https://api.cloudinary.com/v1_1/${CLOUD_NAME}/auto/upload`,
      {
        method: 'POST',
        body: formData,
      }
    );

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Cloudinary upload failed: ${error.error?.message || 'Unknown error'}`);
    }

    const data = await response.json();
    console.log('Cloudinary upload successful:', data);

    return {
      url: data.url,
      publicId: data.public_id,
      format: data.format,
      bytes: data.bytes,
      secureUrl: data.secure_url,
    };
  } catch (error) {
    console.error('Cloudinary upload error:', error);
    throw error;
  }
}

/**
 * Delete file from Cloudinary
 * Note: Requires server-side implementation with API secret
 * For now, we'll keep files (they're in free tier anyway)
 */
export async function deleteFromCloudinary(publicId: string): Promise<void> {
  console.warn('Cloudinary delete requires server-side implementation');
  // TODO: Implement server-side delete endpoint if needed
}

/**
 * Get Cloudinary URL for a public ID
 */
export function getCloudinaryUrl(publicId: string): string {
  return `https://res.cloudinary.com/${CLOUD_NAME}/raw/upload/${publicId}`;
}
