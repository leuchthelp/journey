const COLUMN_NAME = "name";
const COLUMN_JSON = "details";

export function deleteAllIn(input: string) {
  let query = `DELETE FROM ${input}`;
  return query;
}

export function ensureTableWithJson(input: string) {
  let query =
    `CREATE TABLE if NOT EXISTS ${input} (` +
    "id INTEGER PRIMARY KEY," +
    `${COLUMN_NAME} TEXT NOT NULL,` +
    `${COLUMN_JSON} TEXT` +
    ");";
  return query;
}

export function insertNJsonIn(input: string) {
  let query = `INSERT into ${input} (${COLUMN_NAME}, ${COLUMN_JSON}) VALUES ($1, $2)`;
  return query;
}

export function selectNJsonIn(input: string, selector: string) {
  let query =
    `SELECT json_extract(${COLUMN_JSON}, '$.${selector}') AS ${selector} ` +
    `FROM ${input};`;
  return query;
}
