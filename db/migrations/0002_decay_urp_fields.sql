-- Backfill URP priority fields for legacy/default priority JSON shapes.
-- This avoids silent data drift for nodes created with older defaults.

UPDATE nodes
SET priority = json_patch(
    json_replace(
        json_remove(json_remove(priority, '$.age_days'), '$.access_count_30d'),
        '$.access_count_90d', json_extract(priority, '$.access_count_90d')
    ),
    '{"access_count_30active":0,"access_count_90active":0,"today_touches":0,"access_history":[],"link_count":0}'
)
WHERE json_extract(priority, '$.access_count_30active') IS NULL;
