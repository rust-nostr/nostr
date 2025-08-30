-- groups: drop image_url, add image_hash (BLOB)
ALTER TABLE groups DROP COLUMN image_url;
ALTER TABLE groups ADD COLUMN image_hash BLOB;

-- welcomes: drop group_image_url, add group_image_hash (BLOB)
ALTER TABLE welcomes DROP COLUMN group_image_url;
ALTER TABLE welcomes ADD COLUMN group_image_hash BLOB;
