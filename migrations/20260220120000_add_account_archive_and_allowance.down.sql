DROP INDEX IF EXISTS idx_account_archived;

ALTER TABLE account
    DROP COLUMN IF EXISTS next_transfer_amount,
    DROP COLUMN IF EXISTS is_archived;
