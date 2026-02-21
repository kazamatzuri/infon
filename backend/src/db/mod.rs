// Database access layer (SQLite via sqlx).

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bot {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BotVersion {
    pub id: i64,
    pub bot_id: i64,
    pub version: i32,
    pub code: String,
    pub api_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tournament {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub map: String,
    pub config: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TournamentEntry {
    pub id: i64,
    pub tournament_id: i64,
    pub bot_version_id: i64,
    pub slot_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TournamentResult {
    pub id: i64,
    pub tournament_id: i64,
    pub player_slot: i32,
    pub bot_version_id: i64,
    pub final_score: i32,
    pub creatures_spawned: i32,
    pub creatures_killed: i32,
    pub creatures_lost: i32,
    pub finished_at: String,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bot_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bot_id INTEGER NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
                version INTEGER NOT NULL,
                code TEXT NOT NULL,
                api_type TEXT NOT NULL DEFAULT 'oo',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(bot_id, version)
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tournaments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'created',
                map TEXT NOT NULL DEFAULT 'default',
                config TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tournament_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
                bot_version_id INTEGER NOT NULL REFERENCES bot_versions(id),
                slot_name TEXT NOT NULL DEFAULT ''
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tournament_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tournament_id INTEGER NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
                player_slot INTEGER NOT NULL,
                bot_version_id INTEGER NOT NULL REFERENCES bot_versions(id),
                final_score INTEGER NOT NULL DEFAULT 0,
                creatures_spawned INTEGER NOT NULL DEFAULT 0,
                creatures_killed INTEGER NOT NULL DEFAULT 0,
                creatures_lost INTEGER NOT NULL DEFAULT 0,
                finished_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ── Bot CRUD ──────────────────────────────────────────────────────

    pub async fn create_bot(&self, name: &str, description: &str) -> Result<Bot, sqlx::Error> {
        let row = sqlx::query_as::<_, Bot>(
            "INSERT INTO bots (name, description) VALUES (?, ?) RETURNING id, name, description, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_bots(&self) -> Result<Vec<Bot>, sqlx::Error> {
        let rows =
            sqlx::query_as::<_, Bot>("SELECT id, name, description, created_at, updated_at FROM bots ORDER BY id")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    pub async fn get_bot(&self, id: i64) -> Result<Option<Bot>, sqlx::Error> {
        let row = sqlx::query_as::<_, Bot>(
            "SELECT id, name, description, created_at, updated_at FROM bots WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_bot(
        &self,
        id: i64,
        name: &str,
        description: &str,
    ) -> Result<Option<Bot>, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bots SET name = ?, description = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(name)
        .bind(description)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_bot(id).await
    }

    pub async fn delete_bot(&self, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM bots WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ── Bot Versions ──────────────────────────────────────────────────

    pub async fn create_bot_version(
        &self,
        bot_id: i64,
        code: &str,
        api_type: &str,
    ) -> Result<BotVersion, sqlx::Error> {
        // Determine next version number for this bot
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version) FROM bot_versions WHERE bot_id = ?",
        )
        .bind(bot_id)
        .fetch_one(&self.pool)
        .await?;

        let next_version = max_version.unwrap_or(0) + 1;

        let row = sqlx::query_as::<_, BotVersion>(
            "INSERT INTO bot_versions (bot_id, version, code, api_type) VALUES (?, ?, ?, ?) RETURNING id, bot_id, version, code, api_type, created_at",
        )
        .bind(bot_id)
        .bind(next_version)
        .bind(code)
        .bind(api_type)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_bot_versions(&self, bot_id: i64) -> Result<Vec<BotVersion>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BotVersion>(
            "SELECT id, bot_id, version, code, api_type, created_at FROM bot_versions WHERE bot_id = ? ORDER BY version",
        )
        .bind(bot_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_bot_version(
        &self,
        bot_id: i64,
        version_id: i64,
    ) -> Result<Option<BotVersion>, sqlx::Error> {
        let row = sqlx::query_as::<_, BotVersion>(
            "SELECT id, bot_id, version, code, api_type, created_at FROM bot_versions WHERE bot_id = ? AND id = ?",
        )
        .bind(bot_id)
        .bind(version_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    // ── Tournament CRUD ───────────────────────────────────────────────

    pub async fn create_tournament(
        &self,
        name: &str,
        map: &str,
    ) -> Result<Tournament, sqlx::Error> {
        let row = sqlx::query_as::<_, Tournament>(
            "INSERT INTO tournaments (name, map) VALUES (?, ?) RETURNING id, name, status, map, config, created_at",
        )
        .bind(name)
        .bind(map)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_tournaments(&self) -> Result<Vec<Tournament>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Tournament>(
            "SELECT id, name, status, map, config, created_at FROM tournaments ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_tournament(&self, id: i64) -> Result<Option<Tournament>, sqlx::Error> {
        let row = sqlx::query_as::<_, Tournament>(
            "SELECT id, name, status, map, config, created_at FROM tournaments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_tournament_status(
        &self,
        id: i64,
        status: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE tournaments SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ── Tournament Entries ────────────────────────────────────────────

    pub async fn add_tournament_entry(
        &self,
        tournament_id: i64,
        bot_version_id: i64,
        slot_name: &str,
    ) -> Result<TournamentEntry, sqlx::Error> {
        let row = sqlx::query_as::<_, TournamentEntry>(
            "INSERT INTO tournament_entries (tournament_id, bot_version_id, slot_name) VALUES (?, ?, ?) RETURNING id, tournament_id, bot_version_id, slot_name",
        )
        .bind(tournament_id)
        .bind(bot_version_id)
        .bind(slot_name)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_tournament_entries(
        &self,
        tournament_id: i64,
    ) -> Result<Vec<TournamentEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TournamentEntry>(
            "SELECT id, tournament_id, bot_version_id, slot_name FROM tournament_entries WHERE tournament_id = ? ORDER BY id",
        )
        .bind(tournament_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn remove_tournament_entry(&self, entry_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tournament_entries WHERE id = ?")
            .bind(entry_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ── Tournament Results ────────────────────────────────────────────

    pub async fn add_tournament_result(
        &self,
        tournament_id: i64,
        player_slot: i32,
        bot_version_id: i64,
        final_score: i32,
        creatures_spawned: i32,
        creatures_killed: i32,
        creatures_lost: i32,
    ) -> Result<TournamentResult, sqlx::Error> {
        let row = sqlx::query_as::<_, TournamentResult>(
            "INSERT INTO tournament_results (tournament_id, player_slot, bot_version_id, final_score, creatures_spawned, creatures_killed, creatures_lost) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, tournament_id, player_slot, bot_version_id, final_score, creatures_spawned, creatures_killed, creatures_lost, finished_at",
        )
        .bind(tournament_id)
        .bind(player_slot)
        .bind(bot_version_id)
        .bind(final_score)
        .bind(creatures_spawned)
        .bind(creatures_killed)
        .bind(creatures_lost)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_tournament_results(
        &self,
        tournament_id: i64,
    ) -> Result<Vec<TournamentResult>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TournamentResult>(
            "SELECT id, tournament_id, player_slot, bot_version_id, final_score, creatures_spawned, creatures_killed, creatures_lost, finished_at FROM tournament_results WHERE tournament_id = ? ORDER BY final_score DESC",
        )
        .bind(tournament_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_db() -> Database {
        Database::new("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_list_bots() {
        let db = test_db().await;

        let bot1 = db.create_bot("Bot1", "First bot").await.unwrap();
        assert_eq!(bot1.name, "Bot1");
        assert_eq!(bot1.description, "First bot");

        let bot2 = db.create_bot("Bot2", "Second bot").await.unwrap();
        assert_eq!(bot2.name, "Bot2");

        let bots = db.list_bots().await.unwrap();
        assert_eq!(bots.len(), 2);
        assert_eq!(bots[0].name, "Bot1");
        assert_eq!(bots[1].name, "Bot2");

        let fetched = db.get_bot(bot1.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "Bot1");

        let missing = db.get_bot(999).await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_update_bot() {
        let db = test_db().await;

        let bot = db.create_bot("Original", "desc").await.unwrap();
        let updated = db
            .update_bot(bot.id, "Updated", "new desc")
            .await
            .unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.description, "new desc");

        let not_found = db.update_bot(999, "X", "Y").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_delete_bot() {
        let db = test_db().await;

        let bot = db.create_bot("ToDelete", "").await.unwrap();
        assert!(db.delete_bot(bot.id).await.unwrap());
        assert!(!db.delete_bot(bot.id).await.unwrap());

        let bots = db.list_bots().await.unwrap();
        assert!(bots.is_empty());
    }

    #[tokio::test]
    async fn test_bot_versions() {
        let db = test_db().await;

        let bot = db.create_bot("VersionBot", "").await.unwrap();

        let v1 = db
            .create_bot_version(bot.id, "print('v1')", "oo")
            .await
            .unwrap();
        assert_eq!(v1.version, 1);
        assert_eq!(v1.code, "print('v1')");
        assert_eq!(v1.api_type, "oo");

        let v2 = db
            .create_bot_version(bot.id, "print('v2')", "state")
            .await
            .unwrap();
        assert_eq!(v2.version, 2);

        let versions = db.list_bot_versions(bot.id).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[1].version, 2);

        let fetched = db.get_bot_version(bot.id, v1.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().code, "print('v1')");
    }

    #[tokio::test]
    async fn test_tournament_crud() {
        let db = test_db().await;

        let t = db.create_tournament("Tourney1", "desert").await.unwrap();
        assert_eq!(t.name, "Tourney1");
        assert_eq!(t.status, "created");
        assert_eq!(t.map, "desert");

        let tournaments = db.list_tournaments().await.unwrap();
        assert_eq!(tournaments.len(), 1);

        let fetched = db.get_tournament(t.id).await.unwrap();
        assert!(fetched.is_some());

        assert!(db
            .update_tournament_status(t.id, "running")
            .await
            .unwrap());
        let updated = db.get_tournament(t.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "running");

        assert!(!db.update_tournament_status(999, "running").await.unwrap());
    }

    #[tokio::test]
    async fn test_tournament_entries() {
        let db = test_db().await;

        let bot = db.create_bot("EntryBot", "").await.unwrap();
        let v = db
            .create_bot_version(bot.id, "code", "oo")
            .await
            .unwrap();
        let t = db.create_tournament("T", "default").await.unwrap();

        let entry = db
            .add_tournament_entry(t.id, v.id, "player1")
            .await
            .unwrap();
        assert_eq!(entry.slot_name, "player1");
        assert_eq!(entry.tournament_id, t.id);

        let entries = db.list_tournament_entries(t.id).await.unwrap();
        assert_eq!(entries.len(), 1);

        assert!(db.remove_tournament_entry(entry.id).await.unwrap());
        assert!(!db.remove_tournament_entry(entry.id).await.unwrap());

        let entries = db.list_tournament_entries(t.id).await.unwrap();
        assert!(entries.is_empty());
    }
}
