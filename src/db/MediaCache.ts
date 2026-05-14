import {
  ensureTableWithJson,
  deleteAllIn,
  insertNJsonIn,
  selectNJsonIn,
  selectNInJsonIn,
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
    return this.database.execute(ensureTableWithJson(this.name_table));
  };

  public deleteAll = async (): Promise<any> => {
    this.database = await this.database;
    return this.database.execute(deleteAllIn(this.name_table));
  };

  public addEntries = async (entries: string[]): Promise<any> => {
    this.database = await this.database;
    return this.database.execute(insertNJsonIn(this.name_table), entries);
  };

  public getEntries = async (): Promise<Object[]> => {
    this.database = await this.database;
    return this.database.select(selectNJsonIn(this.name_table));
  };

  public getWithinEntries = async (selector: string): Promise<Object[]> => {
    this.database = await this.database;
    return this.database.select(selectNInJsonIn(this.name_table, selector));
  };
}
