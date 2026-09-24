CREATE TYPE map_status AS ENUM ('draft', 'complete');

-- Existing maps are complete; new maps start as draft until confirmed
ALTER TABLE maps ADD COLUMN status map_status NOT NULL DEFAULT 'complete';

-- Draft maps may not have a boundary yet
ALTER TABLE maps ALTER COLUMN bounds DROP NOT NULL;
