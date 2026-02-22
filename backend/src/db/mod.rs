// Database access layer (SQLite via sqlx).

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bot {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub owner_id: Option<i64>,
    pub visibility: String,
    pub active_version_id: Option<i64>,
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
    pub is_archived: bool,
    pub elo_rating: i32,
    pub elo_1v1: i32,
    pub elo_peak: i32,
    pub games_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
    pub ffa_placement_points: i32,
    pub ffa_games: i32,
    pub creatures_spawned: i32,
    pub creatures_killed: i32,
    pub creatures_lost: i32,
    pub total_score: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Match {
    pub id: i64,
    pub format: String,
    pub map: String,
    pub status: String,
    pub winner_bot_version_id: Option<i64>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MatchParticipant {
    pub id: i64,
    pub match_id: i64,
    pub bot_version_id: i64,
    pub player_slot: i32,
    pub final_score: i32,
    pub placement: Option<i32>,
    pub elo_before: Option<i32>,
    pub elo_after: Option<i32>,
    pub creatures_spawned: i32,
    pub creatures_killed: i32,
    pub creatures_lost: i32,
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
        // Users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT,
                display_name TEXT,
                avatar_url TEXT,
                bio TEXT,
                role TEXT NOT NULL DEFAULT 'user',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                owner_id INTEGER REFERENCES users(id),
                visibility TEXT NOT NULL DEFAULT 'public',
                active_version_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Add columns to existing bots table if missing
        let _ = sqlx::query("ALTER TABLE bots ADD COLUMN owner_id INTEGER REFERENCES users(id)")
            .execute(&self.pool)
            .await;
        let _ =
            sqlx::query("ALTER TABLE bots ADD COLUMN visibility TEXT NOT NULL DEFAULT 'public'")
                .execute(&self.pool)
                .await;
        let _ = sqlx::query("ALTER TABLE bots ADD COLUMN active_version_id INTEGER")
            .execute(&self.pool)
            .await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bot_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bot_id INTEGER NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
                version INTEGER NOT NULL,
                code TEXT NOT NULL,
                api_type TEXT NOT NULL DEFAULT 'oo',
                is_archived INTEGER NOT NULL DEFAULT 0,
                elo_rating INTEGER NOT NULL DEFAULT 1500,
                elo_1v1 INTEGER NOT NULL DEFAULT 1500,
                elo_peak INTEGER NOT NULL DEFAULT 1500,
                games_played INTEGER NOT NULL DEFAULT 0,
                wins INTEGER NOT NULL DEFAULT 0,
                losses INTEGER NOT NULL DEFAULT 0,
                draws INTEGER NOT NULL DEFAULT 0,
                ffa_placement_points INTEGER NOT NULL DEFAULT 0,
                ffa_games INTEGER NOT NULL DEFAULT 0,
                creatures_spawned INTEGER NOT NULL DEFAULT 0,
                creatures_killed INTEGER NOT NULL DEFAULT 0,
                creatures_lost INTEGER NOT NULL DEFAULT 0,
                total_score INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(bot_id, version)
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        // Add new columns to existing bot_versions if missing
        for col in &[
            "is_archived INTEGER NOT NULL DEFAULT 0",
            "elo_rating INTEGER NOT NULL DEFAULT 1500",
            "elo_1v1 INTEGER NOT NULL DEFAULT 1500",
            "elo_peak INTEGER NOT NULL DEFAULT 1500",
            "games_played INTEGER NOT NULL DEFAULT 0",
            "wins INTEGER NOT NULL DEFAULT 0",
            "losses INTEGER NOT NULL DEFAULT 0",
            "draws INTEGER NOT NULL DEFAULT 0",
            "ffa_placement_points INTEGER NOT NULL DEFAULT 0",
            "ffa_games INTEGER NOT NULL DEFAULT 0",
            "creatures_spawned INTEGER NOT NULL DEFAULT 0",
            "creatures_killed INTEGER NOT NULL DEFAULT 0",
            "creatures_lost INTEGER NOT NULL DEFAULT 0",
            "total_score INTEGER NOT NULL DEFAULT 0",
        ] {
            let _ = sqlx::query(&format!("ALTER TABLE bot_versions ADD COLUMN {col}"))
                .execute(&self.pool)
                .await;
        }

        // Matches table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                format TEXT NOT NULL DEFAULT '1v1',
                map TEXT NOT NULL DEFAULT 'random',
                status TEXT NOT NULL DEFAULT 'pending',
                winner_bot_version_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                finished_at TEXT
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS match_participants (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
                bot_version_id INTEGER NOT NULL REFERENCES bot_versions(id),
                player_slot INTEGER NOT NULL,
                final_score INTEGER NOT NULL DEFAULT 0,
                placement INTEGER,
                elo_before INTEGER,
                elo_after INTEGER,
                creatures_spawned INTEGER NOT NULL DEFAULT 0,
                creatures_killed INTEGER NOT NULL DEFAULT 0,
                creatures_lost INTEGER NOT NULL DEFAULT 0
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

    // ── User CRUD ─────────────────────────────────────────────────────

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, display_name) VALUES (?, ?, ?, ?) RETURNING id, username, email, password_hash, display_name, avatar_url, bio, role, created_at, updated_at",
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_user(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, display_name, avatar_url, bio, role, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, User>(
            "SELECT id, username, email, password_hash, display_name, avatar_url, bio, role, created_at, updated_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_user(
        &self,
        id: i64,
        display_name: Option<&str>,
        bio: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE users SET display_name = COALESCE(?, display_name), bio = COALESCE(?, bio), updated_at = datetime('now') WHERE id = ?",
        )
        .bind(display_name)
        .bind(bio)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.get_user(id).await
    }

    // ── Bot CRUD ──────────────────────────────────────────────────────

    pub async fn create_bot(
        &self,
        name: &str,
        description: &str,
        owner_id: Option<i64>,
    ) -> Result<Bot, sqlx::Error> {
        let row = sqlx::query_as::<_, Bot>(
            "INSERT INTO bots (name, description, owner_id) VALUES (?, ?, ?) RETURNING id, name, description, owner_id, visibility, active_version_id, created_at, updated_at",
        )
        .bind(name)
        .bind(description)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_bots(&self) -> Result<Vec<Bot>, sqlx::Error> {
        let rows =
            sqlx::query_as::<_, Bot>("SELECT id, name, description, owner_id, visibility, active_version_id, created_at, updated_at FROM bots ORDER BY id")
                .fetch_all(&self.pool)
                .await?;
        Ok(rows)
    }

    pub async fn list_bots_by_owner(&self, owner_id: i64) -> Result<Vec<Bot>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Bot>(
            "SELECT id, name, description, owner_id, visibility, active_version_id, created_at, updated_at FROM bots WHERE owner_id = ? ORDER BY id",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_bot(&self, id: i64) -> Result<Option<Bot>, sqlx::Error> {
        let row = sqlx::query_as::<_, Bot>(
            "SELECT id, name, description, owner_id, visibility, active_version_id, created_at, updated_at FROM bots WHERE id = ?",
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

    pub async fn update_bot_visibility(
        &self,
        id: i64,
        visibility: &str,
    ) -> Result<Option<Bot>, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bots SET visibility = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(visibility)
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
        let max_version: Option<i32> =
            sqlx::query_scalar("SELECT MAX(version) FROM bot_versions WHERE bot_id = ?")
                .bind(bot_id)
                .fetch_one(&self.pool)
                .await?;

        let next_version = max_version.unwrap_or(0) + 1;

        let row = sqlx::query_as::<_, BotVersion>(
            "INSERT INTO bot_versions (bot_id, version, code, api_type) VALUES (?, ?, ?, ?) RETURNING id, bot_id, version, code, api_type, is_archived, elo_rating, elo_1v1, elo_peak, games_played, wins, losses, draws, ffa_placement_points, ffa_games, creatures_spawned, creatures_killed, creatures_lost, total_score, created_at",
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
            "SELECT id, bot_id, version, code, api_type, is_archived, elo_rating, elo_1v1, elo_peak, games_played, wins, losses, draws, ffa_placement_points, ffa_games, creatures_spawned, creatures_killed, creatures_lost, total_score, created_at FROM bot_versions WHERE bot_id = ? ORDER BY version",
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
            "SELECT id, bot_id, version, code, api_type, is_archived, elo_rating, elo_1v1, elo_peak, games_played, wins, losses, draws, ffa_placement_points, ffa_games, creatures_spawned, creatures_killed, creatures_lost, total_score, created_at FROM bot_versions WHERE bot_id = ? AND id = ?",
        )
        .bind(bot_id)
        .bind(version_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Get a bot version by its ID alone (without needing bot_id).
    pub async fn get_bot_version_by_id(
        &self,
        version_id: i64,
    ) -> Result<Option<BotVersion>, sqlx::Error> {
        let row = sqlx::query_as::<_, BotVersion>(
            "SELECT id, bot_id, version, code, api_type, is_archived, elo_rating, elo_1v1, elo_peak, games_played, wins, losses, draws, ffa_placement_points, ffa_games, creatures_spawned, creatures_killed, creatures_lost, total_score, created_at FROM bot_versions WHERE id = ?",
        )
        .bind(version_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    // ── Bot Version Management ──────────────────────────────────────

    pub async fn set_active_version(
        &self,
        bot_id: i64,
        version_id: i64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bots SET active_version_id = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(version_id)
        .bind(bot_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn archive_version(
        &self,
        version_id: i64,
        archived: bool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE bot_versions SET is_archived = ? WHERE id = ?")
            .bind(archived)
            .bind(version_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_active_version(&self, bot_id: i64) -> Result<Option<BotVersion>, sqlx::Error> {
        let row = sqlx::query_as::<_, BotVersion>(
            "SELECT bv.id, bv.bot_id, bv.version, bv.code, bv.api_type, bv.is_archived, bv.elo_rating, bv.elo_1v1, bv.elo_peak, bv.games_played, bv.wins, bv.losses, bv.draws, bv.ffa_placement_points, bv.ffa_games, bv.creatures_spawned, bv.creatures_killed, bv.creatures_lost, bv.total_score, bv.created_at FROM bot_versions bv JOIN bots b ON b.active_version_id = bv.id WHERE b.id = ?",
        )
        .bind(bot_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    // ── Match Recording ──────────────────────────────────────────────

    pub async fn create_match(&self, format: &str, map: &str) -> Result<Match, sqlx::Error> {
        let row = sqlx::query_as::<_, Match>(
            "INSERT INTO matches (format, map, status) VALUES (?, ?, 'running') RETURNING id, format, map, status, winner_bot_version_id, created_at, finished_at",
        )
        .bind(format)
        .bind(map)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn finish_match(
        &self,
        match_id: i64,
        winner_bot_version_id: Option<i64>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE matches SET status = 'finished', winner_bot_version_id = ?, finished_at = datetime('now') WHERE id = ?",
        )
        .bind(winner_bot_version_id)
        .bind(match_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_match(&self, id: i64) -> Result<Option<Match>, sqlx::Error> {
        let row = sqlx::query_as::<_, Match>(
            "SELECT id, format, map, status, winner_bot_version_id, created_at, finished_at FROM matches WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn add_match_participant(
        &self,
        match_id: i64,
        bot_version_id: i64,
        player_slot: i32,
    ) -> Result<MatchParticipant, sqlx::Error> {
        let row = sqlx::query_as::<_, MatchParticipant>(
            "INSERT INTO match_participants (match_id, bot_version_id, player_slot) VALUES (?, ?, ?) RETURNING id, match_id, bot_version_id, player_slot, final_score, placement, elo_before, elo_after, creatures_spawned, creatures_killed, creatures_lost",
        )
        .bind(match_id)
        .bind(bot_version_id)
        .bind(player_slot)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_match_participant(
        &self,
        participant_id: i64,
        final_score: i32,
        placement: Option<i32>,
        elo_before: Option<i32>,
        elo_after: Option<i32>,
        creatures_spawned: i32,
        creatures_killed: i32,
        creatures_lost: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE match_participants SET final_score = ?, placement = ?, elo_before = ?, elo_after = ?, creatures_spawned = ?, creatures_killed = ?, creatures_lost = ? WHERE id = ?",
        )
        .bind(final_score)
        .bind(placement)
        .bind(elo_before)
        .bind(elo_after)
        .bind(creatures_spawned)
        .bind(creatures_killed)
        .bind(creatures_lost)
        .bind(participant_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_match_participants(
        &self,
        match_id: i64,
    ) -> Result<Vec<MatchParticipant>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MatchParticipant>(
            "SELECT id, match_id, bot_version_id, player_slot, final_score, placement, elo_before, elo_after, creatures_spawned, creatures_killed, creatures_lost FROM match_participants WHERE match_id = ? ORDER BY player_slot",
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // ── Elo & Stats Updates ──────────────────────────────────────────

    pub async fn update_version_elo(
        &self,
        version_id: i64,
        new_elo: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bot_versions SET elo_rating = ?, elo_1v1 = ?, elo_peak = MAX(elo_peak, ?) WHERE id = ?",
        )
        .bind(new_elo)
        .bind(new_elo)
        .bind(new_elo)
        .bind(version_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_version_stats(
        &self,
        version_id: i64,
        won: bool,
        lost: bool,
        draw: bool,
        score: i32,
        spawned: i32,
        killed: i32,
        died: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bot_versions SET games_played = games_played + 1, wins = wins + ?, losses = losses + ?, draws = draws + ?, total_score = total_score + ?, creatures_spawned = creatures_spawned + ?, creatures_killed = creatures_killed + ?, creatures_lost = creatures_lost + ? WHERE id = ?",
        )
        .bind(won as i32)
        .bind(lost as i32)
        .bind(draw as i32)
        .bind(score as i64)
        .bind(spawned)
        .bind(killed)
        .bind(died)
        .bind(version_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_version_ffa_stats(
        &self,
        version_id: i64,
        placement_points: i32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE bot_versions SET ffa_games = ffa_games + 1, ffa_placement_points = ffa_placement_points + ? WHERE id = ?",
        )
        .bind(placement_points)
        .bind(version_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_bot_version_stats(
        &self,
        version_id: i64,
    ) -> Result<Option<BotVersion>, sqlx::Error> {
        self.get_bot_version_by_id(version_id).await
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
    async fn test_create_and_get_user() {
        let db = test_db().await;

        let user = db
            .create_user("testuser", "test@example.com", "hashedpw", "Test User")
            .await
            .unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, "user");

        let fetched = db.get_user(user.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().username, "testuser");

        let by_name = db.get_user_by_username("testuser").await.unwrap();
        assert!(by_name.is_some());

        let missing = db.get_user_by_username("nonexistent").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_user_unique_constraints() {
        let db = test_db().await;

        db.create_user("user1", "a@b.com", "hash", "User 1")
            .await
            .unwrap();

        // Duplicate username
        let result = db
            .create_user("user1", "c@d.com", "hash", "User 1 dup")
            .await;
        assert!(result.is_err());

        // Duplicate email
        let result = db.create_user("user2", "a@b.com", "hash", "User 2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_and_list_bots() {
        let db = test_db().await;

        let bot1 = db.create_bot("Bot1", "First bot", None).await.unwrap();
        assert_eq!(bot1.name, "Bot1");
        assert_eq!(bot1.description, "First bot");

        let bot2 = db.create_bot("Bot2", "Second bot", None).await.unwrap();
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
    async fn test_bots_with_owner() {
        let db = test_db().await;

        let user = db
            .create_user("owner", "owner@test.com", "hash", "Owner")
            .await
            .unwrap();

        let bot = db
            .create_bot("OwnedBot", "desc", Some(user.id))
            .await
            .unwrap();
        assert_eq!(bot.owner_id, Some(user.id));
        assert_eq!(bot.visibility, "public");

        let user_bots = db.list_bots_by_owner(user.id).await.unwrap();
        assert_eq!(user_bots.len(), 1);
        assert_eq!(user_bots[0].name, "OwnedBot");
    }

    #[tokio::test]
    async fn test_update_bot() {
        let db = test_db().await;

        let bot = db.create_bot("Original", "desc", None).await.unwrap();
        let updated = db.update_bot(bot.id, "Updated", "new desc").await.unwrap();
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

        let bot = db.create_bot("ToDelete", "", None).await.unwrap();
        assert!(db.delete_bot(bot.id).await.unwrap());
        assert!(!db.delete_bot(bot.id).await.unwrap());

        let bots = db.list_bots().await.unwrap();
        assert!(bots.is_empty());
    }

    #[tokio::test]
    async fn test_bot_versions() {
        let db = test_db().await;

        let bot = db.create_bot("VersionBot", "", None).await.unwrap();

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

        assert!(db.update_tournament_status(t.id, "running").await.unwrap());
        let updated = db.get_tournament(t.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "running");

        assert!(!db.update_tournament_status(999, "running").await.unwrap());
    }

    #[tokio::test]
    async fn test_tournament_entries() {
        let db = test_db().await;

        let bot = db.create_bot("EntryBot", "", None).await.unwrap();
        let v = db.create_bot_version(bot.id, "code", "oo").await.unwrap();
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

    #[tokio::test]
    async fn test_bot_version_elo_and_stats() {
        let db = test_db().await;

        let bot = db.create_bot("EloBot", "", None).await.unwrap();
        let v = db.create_bot_version(bot.id, "code", "oo").await.unwrap();

        assert_eq!(v.elo_rating, 1500);
        assert_eq!(v.games_played, 0);

        db.update_version_elo(v.id, 1520).await.unwrap();
        db.update_version_stats(v.id, true, false, false, 100, 5, 3, 2)
            .await
            .unwrap();

        let updated = db.get_bot_version_by_id(v.id).await.unwrap().unwrap();
        assert_eq!(updated.elo_rating, 1520);
        assert_eq!(updated.elo_peak, 1520);
        assert_eq!(updated.games_played, 1);
        assert_eq!(updated.wins, 1);
        assert_eq!(updated.losses, 0);
        assert_eq!(updated.total_score, 100);
        assert_eq!(updated.creatures_spawned, 5);
        assert_eq!(updated.creatures_killed, 3);
        assert_eq!(updated.creatures_lost, 2);
    }

    #[tokio::test]
    async fn test_active_version() {
        let db = test_db().await;

        let bot = db.create_bot("ActiveBot", "", None).await.unwrap();
        assert!(bot.active_version_id.is_none());

        let v1 = db.create_bot_version(bot.id, "v1", "oo").await.unwrap();
        db.set_active_version(bot.id, v1.id).await.unwrap();

        let bot = db.get_bot(bot.id).await.unwrap().unwrap();
        assert_eq!(bot.active_version_id, Some(v1.id));

        let active = db.get_active_version(bot.id).await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, v1.id);
    }

    #[tokio::test]
    async fn test_version_archiving() {
        let db = test_db().await;

        let bot = db.create_bot("ArchiveBot", "", None).await.unwrap();
        let v = db.create_bot_version(bot.id, "code", "oo").await.unwrap();
        assert!(!v.is_archived);

        db.archive_version(v.id, true).await.unwrap();
        let archived = db.get_bot_version_by_id(v.id).await.unwrap().unwrap();
        assert!(archived.is_archived);

        db.archive_version(v.id, false).await.unwrap();
        let unarchived = db.get_bot_version_by_id(v.id).await.unwrap().unwrap();
        assert!(!unarchived.is_archived);
    }

    #[tokio::test]
    async fn test_match_recording() {
        let db = test_db().await;

        let bot = db.create_bot("MatchBot", "", None).await.unwrap();
        let v1 = db.create_bot_version(bot.id, "code1", "oo").await.unwrap();
        let v2 = db.create_bot_version(bot.id, "code2", "oo").await.unwrap();

        let m = db.create_match("1v1", "random").await.unwrap();
        assert_eq!(m.format, "1v1");
        assert_eq!(m.status, "running");

        let p1 = db.add_match_participant(m.id, v1.id, 0).await.unwrap();
        let p2 = db.add_match_participant(m.id, v2.id, 1).await.unwrap();

        db.update_match_participant(p1.id, 150, Some(1), Some(1500), Some(1520), 10, 5, 3)
            .await
            .unwrap();
        db.update_match_participant(p2.id, 80, Some(2), Some(1500), Some(1480), 8, 3, 5)
            .await
            .unwrap();

        db.finish_match(m.id, Some(v1.id)).await.unwrap();

        let finished = db.get_match(m.id).await.unwrap().unwrap();
        assert_eq!(finished.status, "finished");
        assert_eq!(finished.winner_bot_version_id, Some(v1.id));

        let participants = db.get_match_participants(m.id).await.unwrap();
        assert_eq!(participants.len(), 2);
        assert_eq!(participants[0].final_score, 150);
        assert_eq!(participants[1].final_score, 80);
    }
}
