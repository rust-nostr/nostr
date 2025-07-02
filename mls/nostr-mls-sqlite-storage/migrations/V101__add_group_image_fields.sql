-- Add image_url and image_key to groups table
ALTER TABLE groups ADD COLUMN image_url TEXT;
ALTER TABLE groups ADD COLUMN image_key BLOB;

-- Add group_image_url and image_key to welcomes table
ALTER TABLE welcomes ADD COLUMN group_image_url TEXT;
ALTER TABLE welcomes ADD COLUMN group_image_key BLOB;
