import { GetObjectCommand, PutObjectCommand, S3Client } from '@aws-sdk/client-s3';
import { getSignedUrl } from '@aws-sdk/s3-request-presigner';

const DEFAULT_BUCKET = 'assets';

export function isStorageConfigured(): boolean {
  return Boolean(
    process.env.AWS_ENDPOINT_URL_S3 &&
      process.env.AWS_ACCESS_KEY_ID &&
      process.env.AWS_SECRET_ACCESS_KEY
  );
}

function getS3Client(): S3Client {
  if (!isStorageConfigured()) {
    throw new Error('Neon Object Storage credentials are not configured.');
  }
  return new S3Client({
    endpoint: process.env.AWS_ENDPOINT_URL_S3,
    region: process.env.AWS_REGION || 'us-east-2',
    forcePathStyle: true,
    credentials: {
      accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
    },
  });
}

export async function uploadDocSnapshot(params: {
  docPath: string;
  content: string;
}): Promise<{ objectKey: string; sizeBytes: number }> {
  const s3 = getS3Client();
  const safeName = params.docPath.replace(/[^a-zA-Z0-9._-]/g, '_');
  const objectKey = `snapshots/${Date.now()}-${safeName}`;
  const body = Buffer.from(params.content, 'utf8');

  await s3.send(
    new PutObjectCommand({
      Bucket: DEFAULT_BUCKET,
      Key: objectKey,
      Body: body,
      ContentType: 'text/markdown; charset=utf-8',
    })
  );

  return {
    objectKey,
    sizeBytes: body.byteLength,
  };
}

export async function getSnapshotPresignedUrl(objectKey: string): Promise<string> {
  const s3 = getS3Client();
  return getSignedUrl(
    s3,
    new GetObjectCommand({
      Bucket: DEFAULT_BUCKET,
      Key: objectKey,
    }),
    { expiresIn: 3600 }
  );
}
