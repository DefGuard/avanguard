-- Add up migration script here
ALTER TABLE "refreshtoken" ADD COLUMN ip_address TEXT NOT NULL; 
