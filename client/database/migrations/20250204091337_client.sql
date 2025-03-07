CREATE TABLE Aci (
  aci   TEXT NOT NULL
);

CREATE TABLE Username (
  username  TEXT NOT NULL
);

CREATE TABLE Password (
  password  TEXT NOT NULL
);

CREATE TABLE IdentityKeys (
  id                  INTEGER PRIMARY KEY,
  public_key          TEXT NOT NULL,
  private_key         TEXT NOT NULL,
  registration_id     UNSIGNED BIG INT NOT NULL
);

CREATE TABLE DeviceIdentityKeyStore (
  id               INTEGER PRIMARY KEY,
  address          TEXT NOT NULL UNIQUE,
  identity_key     TEXT NOT NULL    -- identity key for another device
);

CREATE TABLE DevicePreKeyStore (
  id              INTEGER PRIMARY KEY,
  pre_key_id      UNSIGNED BIG INT NOT NULL UNIQUE,
  pre_key_record  TEXT NOT NULL
);

CREATE TABLE DeviceSignedPreKeyStore (
  id                      INTEGER PRIMARY KEY,
  signed_pre_key_id       UNSIGNED BIG INT NOT NULL UNIQUE,
  signed_pre_key_record   TEXT NOT NULL
);

CREATE TABLE DeviceKyberPreKeyStore (
  id                    INTEGER PRIMARY KEY,
  kyber_pre_key_id      UNSIGNED BIG INT NOT NULL UNIQUE,
  kyber_pre_key_record  TEXT NOT NULL
);

CREATE TABLE DeviceSessionStore (
  id              INTEGER PRIMARY KEY,
  address         TEXT NOT NULL UNIQUE,
  session_record  TEXT NOT NULL
);

CREATE TABLE DeviceSenderKeyStore (
  id                  INTEGER PRIMARY KEY,
  address             TEXT NOT NULL UNIQUE,
  sender_key_record   TEXT NOT NULL
);

CREATE TABLE Contacts (
  id INTEGER PRIMARY KEY,
  account_id TEXT NOT NULL,
  device_id  INTEGER NOT NULL
);

CREATE TABLE MessageStore (
  id INTEGER PRIMARY KEY,
  contact_id INTEGER NOT NULL,
  content BLOB NOT NULL,
  FOREIGN KEY (contact_id) REFERENCES Contacts(id)
);