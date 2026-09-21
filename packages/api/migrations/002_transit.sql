CREATE TYPE transit_route_type AS ENUM (
    'bus', 'tram', 'subway', 'train', 'ferry',
    'monorail', 'light_rail', 'trolleybus', 'funicular'
);

CREATE TABLE transit_stops (
    id       UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    map_id   UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    osm_id   BIGINT,
    name     TEXT NOT NULL,
    lat      DOUBLE PRECISION NOT NULL,
    lng      DOUBLE PRECISION NOT NULL
);

CREATE TABLE transit_routes (
    id         UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    map_id     UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    osm_id     BIGINT,
    name       TEXT NOT NULL,
    long_name  TEXT,
    route_type transit_route_type NOT NULL,
    color      TEXT,
    shape      JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE transit_route_stops (
    route_id UUID NOT NULL REFERENCES transit_routes(id) ON DELETE CASCADE,
    stop_id  UUID NOT NULL REFERENCES transit_stops(id) ON DELETE CASCADE,
    seq      INTEGER NOT NULL,
    PRIMARY KEY (route_id, stop_id)
);

CREATE INDEX transit_stops_map_idx  ON transit_stops(map_id);
CREATE INDEX transit_routes_map_idx ON transit_routes(map_id);
