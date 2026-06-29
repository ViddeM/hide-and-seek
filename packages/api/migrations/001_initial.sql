-- A line, defined by two points.
CREATE TABLE line (
    id          UUID DEFAULT gen_random_uuid(),
    start_lat   DOUBLE PRECISION NOT NULL,
    start_lng   DOUBLE PRECISION NOT NULL,
    end_lat     DOUBLE PRECISION NOT NULL,
    end_lng     DOUBLE PRECISION NOT NULL,

    PRIMARY KEY (id)
);

-- A circle, defined by a center point and a radius in meters.
CREATE TABLE circle (
    id              UUID DEFAULT gen_random_uuid(),
    center_lat      DOUBLE PRECISION NOT NULL,
    center_lng      DOUBLE PRECISION NOT NULL,
    radius_meters   INTEGER NOT NULL,

    PRIMARY KEY (id)
);

-- A polygon, a closed shape defined by a series of points in clockwise order.
CREATE TABLE polygon (
    id UUID DEFAULT gen_random_uuid(),

    PRIMARY KEY (id)
);

-- A point in a polygon, used to define the shape of the polygon.
CREATE TABLE polygon_point (
    id          UUID DEFAULT gen_random_uuid(),
    -- The number of the point in the polygon, starting at 0 and increasing clockwise.
    number      INTEGER NOT NULL,
    polygon_id  UUID NOT NULL,
    lat         DOUBLE PRECISION NOT NULL,
    lng         DOUBLE PRECISION NOT NULL,

    PRIMARY KEY (id),

    FOREIGN KEY (polygon_id) REFERENCES polygon(id)
);

-- An area on the map, can be one of a number of different shapes.
CREATE TABLE area (
    id          UUID DEFAULT gen_random_uuid(),
    line_id     UUID NULL,
    circle_id   UUID NULL,
    polygon_id  UUID NULL,

    PRIMARY KEY (id),

    FOREIGN KEY (line_id) REFERENCES line(id),
    FOREIGN KEY (circle_id) REFERENCES circle(id),
    FOREIGN KEY (polygon_id) REFERENCES polygon(id),

    CHECK (
        num_nonnulls(line_id, circle_id, polygon_id) = 1
    )
);

-- Maps (reusable across games)
CREATE TYPE map_size AS ENUM ('small', 'medium', 'large');

CREATE TABLE maps (
    id             UUID DEFAULT gen_random_uuid(),
    name           TEXT NOT NULL,
    size           map_size NOT NULL,
    bounds         UUID NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (id),

    FOREIGN KEY (bounds) REFERENCES polygon(id)
);


-- Games
CREATE TYPE game_status AS ENUM ('lobby', 'active', 'finished');

CREATE TABLE games (
    id          UUID DEFAULT gen_random_uuid(),
    code        CHAR(8) NOT NULL UNIQUE,
    map_id      UUID NOT NULL,
    status      game_status NOT NULL DEFAULT 'lobby',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,

    PRIMARY KEY (id),

    FOREIGN KEY (map_id) REFERENCES maps(id)
);

CREATE INDEX games_code_idx ON games(code);



-- Exclusion zones added by seekers (core real-time data)
CREATE TABLE exclusion_zones (
    id              UUID DEFAULT gen_random_uuid(),
    game_id         UUID NOT NULL,
    area_id         UUID NOT NULL,
    -- Whether the exclusion zone is for the 'outside' of the area (true) or the 'inside' of the area (false).
    -- Outside is defined as the area left of the points in this shape when traversed in clockwise order.
    -- For a line this is 
    exclude_outside BOOLEAN NOT NULL,
    label           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (id),

    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (area_id) REFERENCES area(id)
);

CREATE INDEX exclusion_zones_game_idx ON exclusion_zones(game_id);
