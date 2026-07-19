CREATE TYPE jlpt_level AS ENUM ('n5', 'n4', 'n3', 'n2', 'n1');

CREATE TABLE radical (
  id smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  literal text NOT NULL UNIQUE
);

CREATE TABLE kanji (
  id smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  literal text NOT NULL UNIQUE,
  meanings text[] NOT NULL,
  kunyomi text[] NOT NULL,
  onyomi text[] NOT NULL,
  jlpt jlpt_level
);

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
