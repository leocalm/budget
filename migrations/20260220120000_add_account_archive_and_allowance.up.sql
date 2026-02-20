ALTER TABLE account
    ADD COLUMN is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN next_transfer_amount BIGINT NULL;

CREATE INDEX IF NOT EXISTS idx_account_archived ON account (user_id, is_archived);
