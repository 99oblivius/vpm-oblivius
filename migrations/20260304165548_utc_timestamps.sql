-- Fix existing bare datetime values missing timezone info
UPDATE packages SET created_at = created_at || '+00:00' WHERE created_at NOT LIKE '%+%' AND created_at NOT LIKE '%Z';
UPDATE package_versions SET created_at = created_at || '+00:00' WHERE created_at NOT LIKE '%+%' AND created_at NOT LIKE '%Z';
