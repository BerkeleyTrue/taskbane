use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ring::rand::SecureRandom;
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

fn generate_challenge() -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    let mut challenge = vec![0u8; 32];
    rng.fill(&mut challenge)
        .expect("Failed to generate random challenge");
    challenge
}

fn generate_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredChallenge {
    challenge: Vec<u8>,
    timestamp: u64,
    user_id: Uuid,
}

impl StoredChallenge {
    pub fn new(challenge: Vec<u8>, timestamp: u64, user_id: Uuid) -> Self {
        StoredChallenge {
            challenge,
            timestamp,
            user_id,
        }
    }

    pub fn challenge(&self) -> &[u8] {
        &self.challenge
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
}

#[derive(Debug)]
pub struct ChallangeStore {
    challenges: std::collections::HashMap<Uuid, StoredChallenge>,
}

#[async_trait]
pub trait ChallengeRepository: Send + Sync {
    async fn add(&self, stored_challenge: StoredChallenge) -> Result<StoredChallenge, String>;
}

struct ChallengeMemRepo {
    store: Arc<Mutex<ChallangeStore>>,
}

#[async_trait]
impl ChallengeRepository for ChallengeMemRepo {
    async fn add(&self, stored_challenge: StoredChallenge) -> Result<StoredChallenge, String> {
        let mut store = self.store.lock().await;
        let user_id = stored_challenge.user_id().clone();
        let existing_challange_for_user = store
            .challenges
            .iter()
            .any(move |(uuid, _)| uuid.clone() == user_id);

        if existing_challange_for_user {
            return Err("Existing Challenge for user".to_string());
        }

        let challenge_clone = stored_challenge.clone();
        store
            .challenges
            .insert(challenge_clone.user_id(), challenge_clone);
        Ok(stored_challenge.clone())
    }
}

#[derive(Clone)]
pub struct ChallengeService {
    repo: Arc<dyn ChallengeRepository>,
}

impl ChallengeService {
    pub fn new(repo: Arc<dyn ChallengeRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_challenge(&self, user_id: Uuid) -> Result<StoredChallenge, String> {
        let challenge = generate_challenge();
        let timestamp = generate_timestamp();
        
        let stored_challenge = StoredChallenge {
            challenge,
            timestamp,
            user_id,
        };
        self.repo.add(stored_challenge).await
    }
}

pub fn create_challenge_service() -> ChallengeService {
    let store = Arc::new(Mutex::new(ChallangeStore {
        challenges: HashMap::new()
    }));
    let repo = Arc::new(ChallengeMemRepo {
        store,
    });

    ChallengeService {
        repo,
    }
}
