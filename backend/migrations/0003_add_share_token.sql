ALTER TABLE documents
ADD COLUMN share_token TEXT DEFAULT NULL;

CREATE UNIQUE INDEX idx_documents_share_token ON documents(share_token) WHERE share_token IS NOT NULL;
