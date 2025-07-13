CREATE TABLE protocols (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL CHECK (length(name) > 0),
    program_id VARCHAR(44) NOT NULL CHECK (length(program_id) = 44),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_protocols_name ON protocols(name);
CREATE INDEX idx_protocols_program_id ON protocols(program_id);
CREATE INDEX idx_protocols_is_active ON protocols(is_active);
CREATE INDEX idx_protocols_created_at ON protocols(created_at);
