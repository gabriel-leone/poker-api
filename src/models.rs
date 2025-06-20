use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub chips: u32,
    pub hand: Vec<Card>,
    pub current_bet: u32,
    pub is_folded: bool,
    pub is_all_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameState {
    Waiting,
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Raise(u32),
    AllIn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub community_cards: Vec<Card>,
    pub pot: u32,
    pub current_bet: u32,
    pub current_player_index: usize,
    pub dealer_index: usize,
    pub small_blind: u32,
    pub big_blind: u32,
    pub state: GameState,
    pub round_bets: HashMap<String, u32>,
}

#[derive(Debug)]
pub struct Room {
    pub id: String,
    pub creator_id: String,
    pub players: HashMap<String, Player>,
    pub game: Option<Game>,
    pub max_players: usize,
    pub websocket_senders: HashMap<String, mpsc::UnboundedSender<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub creator_name: String,
    pub max_players: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub player_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomResponse {
    pub success: bool,
    pub message: String,
    pub player_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameActionMessage {
    pub player_id: String,
    pub action: PlayerAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard = 1,
    OnePair = 2,
    TwoPair = 3,
    ThreeOfAKind = 4,
    Straight = 5,
    Flush = 6,
    FullHouse = 7,
    FourOfAKind = 8,
    StraightFlush = 9,
    RoyalFlush = 10,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandEvaluation {
    pub rank: HandRank,
    pub kickers: Vec<u8>, // Cartas que desempatam
    pub cards: Vec<Card>, // As 5 melhores cartas
}

impl PartialEq for HandEvaluation {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.kickers == other.kickers
    }
}

impl Eq for HandEvaluation {}

impl PartialOrd for HandEvaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandEvaluation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.rank.cmp(&other.rank) {
            std::cmp::Ordering::Equal => self.kickers.cmp(&other.kickers),
            other => other,
        }
    }
}
