-- Add image_nonce to groups table
ALTER TABLE groups ADD COLUMN image_nonce BLOB;

-- Add image_nonce to welcomes table
ALTER TABLE welcomes ADD COLUMN group_image_nonce BLOB;