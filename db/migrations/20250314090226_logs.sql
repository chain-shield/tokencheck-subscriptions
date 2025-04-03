CREATE TABLE logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP NOT NULL,
    method VARCHAR(10) NOT NULL,
    path TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    user_id UUID NULL REFERENCES users(id),
    params JSONB,
    request_body JSONB,
    response_body JSONB,
    ip_address INET NOT NULL,
    user_agent VARCHAR(255) NOT NULL
);