-- Maps (reusable across games)
CREATE TYPE map_size AS ENUM ('small', 'medium', 'large');

CREATE TABLE maps (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT NOT NULL,
    size           map_size NOT NULL,
    bounds_sw_lat  DOUBLE PRECISION NOT NULL,
    bounds_sw_lng  DOUBLE PRECISION NOT NULL,
    bounds_ne_lat  DOUBLE PRECISION NOT NULL,
    bounds_ne_lng  DOUBLE PRECISION NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Named stops on the map (train stations, airports, etc.)
CREATE TABLE map_stops (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id    UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    lat       DOUBLE PRECISION NOT NULL,
    lng       DOUBLE PRECISION NOT NULL,
    stop_type TEXT NOT NULL
);

-- Questions available for each map (can include radius-based questions)
CREATE TABLE map_questions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id        UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    text          TEXT NOT NULL,
    radius_m      INTEGER,
    requires_stop BOOLEAN NOT NULL DEFAULT false
);

-- Games
CREATE TYPE game_status AS ENUM ('lobby', 'active', 'finished');

CREATE TABLE games (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code        CHAR(6) NOT NULL UNIQUE,
    map_id      UUID NOT NULL REFERENCES maps(id),
    status      game_status NOT NULL DEFAULT 'lobby',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

CREATE INDEX games_code_idx ON games(code);

-- Teams
CREATE TYPE team_role AS ENUM ('hider', 'seeker');

CREATE TABLE teams (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id  UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name     TEXT NOT NULL,
    role     team_role NOT NULL,
    UNIQUE(game_id, name)
);

-- Players (ephemeral per game session, no persistent accounts)
CREATE TABLE players (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id      UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    is_host      BOOLEAN NOT NULL DEFAULT false,
    joined_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Card definitions (static, seeded in migration 002)
CREATE TYPE card_type AS ENUM ('bonus', 'curse');

CREATE TABLE cards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    card_type   card_type NOT NULL,
    effect      TEXT NOT NULL,
    flavor_text TEXT
);

-- Cards drawn by a team during a game
CREATE TABLE drawn_cards (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id   UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    team_id   UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    card_id   UUID NOT NULL REFERENCES cards(id),
    drawn_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    played_at TIMESTAMPTZ
);

-- Exclusion zones added by seekers (core real-time data)
CREATE TABLE exclusion_zones (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id         UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    team_id         UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    question_id     UUID REFERENCES map_questions(id),
    center_lat      DOUBLE PRECISION NOT NULL,
    center_lng      DOUBLE PRECISION NOT NULL,
    radius_m        INTEGER NOT NULL,
    -- true  = exclude OUTSIDE the circle (answer was "yes, within X km")
    -- false = exclude INSIDE the circle (answer was "no, not within X km")
    exclude_outside BOOLEAN NOT NULL,
    label           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX exclusion_zones_game_idx ON exclusion_zones(game_id);

-- Turn tracking
CREATE TABLE turns (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_id      UUID NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    hiding_team  UUID NOT NULL REFERENCES teams(id),
    turn_number  INTEGER NOT NULL,
    started_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at     TIMESTAMPTZ
);
