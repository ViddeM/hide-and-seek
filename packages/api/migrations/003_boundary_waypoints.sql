-- Replace rectangular bounds with a JSONB polygon of [[lat,lng], ...] waypoints.
ALTER TABLE maps
    ADD COLUMN boundary_points JSONB NOT NULL DEFAULT '[]';

-- Convert existing rows: box corners → 4-point polygon
UPDATE maps SET boundary_points = jsonb_build_array(
    jsonb_build_array(bounds_sw_lat, bounds_sw_lng),
    jsonb_build_array(bounds_sw_lat, bounds_ne_lng),
    jsonb_build_array(bounds_ne_lat, bounds_ne_lng),
    jsonb_build_array(bounds_ne_lat, bounds_sw_lng)
);

ALTER TABLE maps
    DROP COLUMN bounds_sw_lat,
    DROP COLUMN bounds_sw_lng,
    DROP COLUMN bounds_ne_lat,
    DROP COLUMN bounds_ne_lng;
