-- Reverse of 0001_init: drop all tables in dependency order, then the extension

DROP TABLE IF EXISTS budget_period;
DROP TABLE IF EXISTS budget_category;
DROP TABLE IF EXISTS budget;
DROP TABLE IF EXISTS transaction;
DROP TABLE IF EXISTS vendor;
DROP TABLE IF EXISTS category;
DROP TABLE IF EXISTS account;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS currency;

DROP EXTENSION IF EXISTS "pgcrypto";
