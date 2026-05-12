import {
  ensureTableWithJson,
  deleteAllIn,
  insertNJsonIn,
  selectNJsonIn,
} from "../db/Queries.ts";
import Database from "@tauri-apps/plugin-sql";

export class MediaCache {
  private location: string;
  private name_table: string;
  private database: Database | Promise<Database>;

  constructor(location = "sqlite:test.db", name_table = "sources") {
    this.location = location;
    this.database = Database.load(this.location);

    this.name_table = name_table;
    this.init();
  }

  private init = async (): Promise<any> => {
    this.database = await this.database;
    return await this.database.execute(ensureTableWithJson(this.name_table));
  };

  public addEntries = async (entries: string[]): Promise<any> => {
    this.database = await this.database;
    console.log(insertNJsonIn(this.name_table));
    return await this.database.execute(insertNJsonIn(this.name_table), entries);
  };

  public getEntries = async (selector: string): Promise<Array<Object>> => {
    this.database = await this.database;
    console.log(selectNJsonIn(this.name_table, selector));
    return await this.database.select(selectNJsonIn(this.name_table, selector));
  };
}
