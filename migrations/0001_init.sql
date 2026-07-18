CREATE TABLE radical (
  id smallint PRIMARY KEY,
  literal text NOT NULL
);

CREATE TABLE kanji (
  id smallint PRIMARY KEY,
  literal text NOT NULL,
  kunyomi text[] NOT NULL,
  onyomi text[] NOT NULL
);

-- Look up a kanji by its literal; UNIQUE since each character appears once.
CREATE UNIQUE INDEX kanji_literal_idx ON kanji (literal);

CREATE TABLE kanji_radical (
  kanji_id smallint NOT NULL REFERENCES kanji (id),
  radical_id smallint NOT NULL REFERENCES radical (id),
  PRIMARY KEY (kanji_id, radical_id)
);

-- PK indexes kanji_id (leading col); add index for reverse lookups by radical.
CREATE INDEX kanji_radical_radical_id_idx ON kanji_radical (radical_id);

CREATE TABLE vocabulary (
  id int PRIMARY KEY,
  word text NOT NULL
);

CREATE TABLE vocabulary_kanji (
  vocabulary_id int NOT NULL REFERENCES vocabulary (id),
  kanji_id smallint NOT NULL REFERENCES kanji (id),
  PRIMARY KEY (vocabulary_id, kanji_id)
);

-- PK indexes vocabulary_id (leading col); add index for reverse lookups by kanji.
CREATE INDEX vocabulary_kanji_kanji_id_idx ON vocabulary_kanji (kanji_id);
