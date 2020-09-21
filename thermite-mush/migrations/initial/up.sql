CREATE TABLE IF NOT EXISTS attributeflag (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(20) NOT NULL UNIQUE COLLATE NOCASE,
    perm VARCHAR(255) NOT NULL DEFAULT "#TRUE",
    reset_perm VARCHAR(255) NOT NULL DEFAULT "#TRUE",
);

CREATE TABLE IF NOT EXISTS attribute (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,
    creator INTEGER NOT NULL DEFAULT 0,
    data TEXT NULL
);

CREATE TABLE IF NOT EXISTS attribute_flags (
    id INTEGER NOT NULL PRIMARY KEY,
    attribute INTEGER NOT NULL,
    flag INTEGER NOT NULL,
    FOREIGN KEY(attribute) REFERENCES attribute(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(flag) REFERENCES attributeflag(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(attribute, flag)
);

CREATE TABLE IF NOT EXISTS commandflag (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,

);

CREATE TABLE IF NOT EXISTS dbobjtype (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,
    letter VARCHAR(1) NULL UNIQUE COLLATE NOCASE,
);

CREATE TABLE IF NOT EXISTS flag (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE COLLATE NOCASE,
    letter VARCHAR(1) NULL UNIQUE COLLATE NOCASE,
);

CREATE TABLE IF NOT EXISTS flag_dbobjtypes (
    id INTEGER NOT NULL PRIMARY KEY,
    dbobjtype_id INTEGER NOT NULL,
    flag_id INTEGER NOT NULL,
    FOREIGN KEY(dbobjtype_id) REFERENCES dbobjtype(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(flag_id) REFERENCES flag(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(dbobjtype_id, flag_id)
);

CREATE TABLE IF NOT EXISTS dbobj (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL COLLATE NOCASE DEFAULT "GARBAGE",
    parent INTEGER NOT NULL DEFAULT -1,
    dbobjtype_id INTEGER NOT NULL DEFAULT -1,
    location INTEGER NOT NULL DEFAULT -1,
    zone INTEGER NOT NULL DEFAULT -1,
    owner INTEGER NOT NULL DEFAULT -1,
    money INTEGER NOT NULL DEFAULT 0,
    warn_type INTEGER NOT NULL DEFAULT 0,
    creation_datetime INTEGER NOT NULL DEFAULT 0,
    modify_datetime INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(dbobjtype_id) REFERENCES dbobjtype(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE INDEX dbobj_children on dbobj(parent);
CREATE INDEX dbobj_contents on dbobj(location, dbobjtype_id);
CREATE INDEX dbobj_zoned on dbobj(zone);
CREATE INDEX dbobj_belongings on dbobj(owner);

CREATE TABLE IF NOT EXISTS dbobj_flags (
    id INTEGER NOT NULL PRIMARY KEY,
    dbobj_id INTEGER NOT NULL,
    flag_id INTEGER NOT NULL,
    FOREIGN KEY(dbobj_id) REFERENCES dbobj(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(flag_id) REFERENCES flag(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(dbobj_id, flag_id)
);

CREATE TABLE IF NOT EXISTS dbobj_attributes (
    id INTEGER NOT NULL PRIMARY KEY,
    dbobj_id INTEGER NOT NULL,
    attribute_id INTEGER NOT NULL,
    owner INTEGER NOT NULL DEFAULT -1,
    value TEXT NULL,
    FOREIGN KEY(dbobj_id) REFERENCES dbobj(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(attribute_id) REFERENCES attribute(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(dbobj_id, attribute_id)
);

CREATE TABLE IF NOT EXISTS dbobj_attributes_flags (
    id INTEGER NOT NULL PRIMARY KEY,
    dbobj_attribute_id INTEGER NOT NULL,
    attributeflag_id INTEGER NOT NULL,
    FOREIGN KEY(dbobj_attribute_id) REFERENCES dbobj_attributes(id) ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY(attributeflag_id) REFERENCES attributeflag(id) ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE(dbobj_attribute_id, attributeflag_id)
);