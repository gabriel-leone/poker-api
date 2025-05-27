use crate::models::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use uuid::Uuid;

impl Game {
    pub fn new(players: Vec<Player>) -> Self {
        let mut deck = Self::create_deck();
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);

        Self {
            id: Uuid::new_v4().to_string(),
            players,
            deck,
            community_cards: Vec::new(),
            pot: 0,
            current_bet: 0,
            current_player_index: 0,
            dealer_index: 0,
            small_blind: 5,
            big_blind: 10,
            state: GameState::PreFlop,
            round_bets: HashMap::new(),
        }
    }

    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::new();        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
            Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];

        for suit in &suits {
            for rank in &ranks {
                deck.push(Card { suit: *suit, rank: *rank });
            }
        }

        deck
    }

    pub fn start_round(&mut self) {
        // Reset player states
        for player in &mut self.players {
            player.hand.clear();
            player.current_bet = 0;
            player.is_folded = false;
            player.is_all_in = false;
        }

        self.community_cards.clear();
        self.pot = 0;
        self.current_bet = 0;
        self.round_bets.clear();
        self.state = GameState::PreFlop;

        // Deal cards
        self.deal_hole_cards();

        // Post blinds
        self.post_blinds();

        // Set current player (left of big blind)
        self.current_player_index = (self.dealer_index + 3) % self.players.len();
    }

    fn deal_hole_cards(&mut self) {
        for _ in 0..2 {
            for player in &mut self.players {
                if let Some(card) = self.deck.pop() {
                    player.hand.push(card);
                }
            }
        }
    }

    fn post_blinds(&mut self) {
        let small_blind_index = (self.dealer_index + 1) % self.players.len();
        let big_blind_index = (self.dealer_index + 2) % self.players.len();

        // Small blind
        let small_blind_amount = std::cmp::min(self.small_blind, self.players[small_blind_index].chips);
        self.players[small_blind_index].chips -= small_blind_amount;
        self.players[small_blind_index].current_bet = small_blind_amount;
        self.pot += small_blind_amount;
        self.round_bets.insert(self.players[small_blind_index].id.clone(), small_blind_amount);

        // Big blind
        let big_blind_amount = std::cmp::min(self.big_blind, self.players[big_blind_index].chips);
        self.players[big_blind_index].chips -= big_blind_amount;
        self.players[big_blind_index].current_bet = big_blind_amount;
        self.pot += big_blind_amount;
        self.current_bet = big_blind_amount;
        self.round_bets.insert(self.players[big_blind_index].id.clone(), big_blind_amount);
    }

    pub fn process_action(&mut self, player_id: &str, action: PlayerAction) -> Result<(), String> {
        let current_player = &self.players[self.current_player_index];
        if current_player.id != player_id {
            return Err("Não é sua vez de jogar".to_string());
        }

        if current_player.is_folded {
            return Err("Jogador já foldou".to_string());
        }

        match action {
            PlayerAction::Fold => {
                self.players[self.current_player_index].is_folded = true;
            }
            PlayerAction::Check => {
                if self.current_bet > self.players[self.current_player_index].current_bet {
                    return Err("Não é possível dar check, há uma aposta a ser igualada".to_string());
                }
            }
            PlayerAction::Call => {
                let to_call = self.current_bet - self.players[self.current_player_index].current_bet;
                let available_chips = self.players[self.current_player_index].chips;
                let call_amount = std::cmp::min(to_call, available_chips);

                self.players[self.current_player_index].chips -= call_amount;
                self.players[self.current_player_index].current_bet += call_amount;
                self.pot += call_amount;

                let current_total_bet = *self.round_bets.get(&self.players[self.current_player_index].id).unwrap_or(&0) + call_amount;
                self.round_bets.insert(self.players[self.current_player_index].id.clone(), current_total_bet);

                if self.players[self.current_player_index].chips == 0 {
                    self.players[self.current_player_index].is_all_in = true;
                }
            }
            PlayerAction::Raise(amount) => {
                let to_call = self.current_bet - self.players[self.current_player_index].current_bet;
                let total_bet = to_call + amount;
                let available_chips = self.players[self.current_player_index].chips;

                if total_bet > available_chips {
                    return Err("Fichas insuficientes para essa aposta".to_string());
                }

                self.players[self.current_player_index].chips -= total_bet;
                self.players[self.current_player_index].current_bet += total_bet;
                self.current_bet = self.players[self.current_player_index].current_bet;
                self.pot += total_bet;

                let current_total_bet = *self.round_bets.get(&self.players[self.current_player_index].id).unwrap_or(&0) + total_bet;
                self.round_bets.insert(self.players[self.current_player_index].id.clone(), current_total_bet);
            }
            PlayerAction::AllIn => {
                let all_in_amount = self.players[self.current_player_index].chips;
                self.players[self.current_player_index].chips = 0;
                self.players[self.current_player_index].current_bet += all_in_amount;
                self.players[self.current_player_index].is_all_in = true;
                self.pot += all_in_amount;

                let current_total_bet = *self.round_bets.get(&self.players[self.current_player_index].id).unwrap_or(&0) + all_in_amount;
                self.round_bets.insert(self.players[self.current_player_index].id.clone(), current_total_bet);

                if self.players[self.current_player_index].current_bet > self.current_bet {
                    self.current_bet = self.players[self.current_player_index].current_bet;
                }
            }
        }

        self.next_player();
        self.check_round_completion();

        Ok(())
    }

    fn next_player(&mut self) {
        loop {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
            let player = &self.players[self.current_player_index];
            
            if !player.is_folded && !player.is_all_in {
                break;
            }

            // Se voltamos ao dealer, a rodada acabou
            if self.current_player_index == self.dealer_index {
                self.advance_game_state();
                break;
            }
        }
    }

    fn check_round_completion(&mut self) {
        let active_players: Vec<_> = self.players.iter()
            .filter(|p| !p.is_folded && !p.is_all_in)
            .collect();

        if active_players.len() <= 1 {
            self.advance_game_state();
            return;
        }

        // Verificar se todos os jogadores ativos fizeram a mesma aposta
        let current_bet = self.current_bet;
        let all_bets_equal = active_players.iter()
            .all(|p| p.current_bet == current_bet);

        if all_bets_equal {
            self.advance_game_state();
        }
    }

    fn advance_game_state(&mut self) {
        // Reset current bets for next round
        for player in &mut self.players {
            player.current_bet = 0;
        }
        self.current_bet = 0;
        self.round_bets.clear();

        match self.state {
            GameState::PreFlop => {
                self.state = GameState::Flop;
                self.deal_flop();
            }
            GameState::Flop => {
                self.state = GameState::Turn;
                self.deal_turn();
            }
            GameState::Turn => {
                self.state = GameState::River;
                self.deal_river();
            }
            GameState::River => {
                self.state = GameState::Showdown;
                self.determine_winner();
            }
            _ => {}
        }

        // Reset current player to left of dealer
        self.current_player_index = (self.dealer_index + 1) % self.players.len();
    }

    fn deal_flop(&mut self) {
        // Burn one card
        self.deck.pop();
        
        // Deal 3 community cards
        for _ in 0..3 {
            if let Some(card) = self.deck.pop() {
                self.community_cards.push(card);
            }
        }
    }

    fn deal_turn(&mut self) {
        // Burn one card
        self.deck.pop();
        
        // Deal 1 community card
        if let Some(card) = self.deck.pop() {
            self.community_cards.push(card);
        }
    }

    fn deal_river(&mut self) {
        // Burn one card
        self.deck.pop();
        
        // Deal 1 community card
        if let Some(card) = self.deck.pop() {
            self.community_cards.push(card);
        }
    }

    fn determine_winner(&mut self) {
        // Implementação simplificada - o jogador que não foldou ganha
        let active_players: Vec<_> = self.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded)
            .collect();

        if active_players.len() == 1 {
            let (winner_index, _) = active_players[0];
            self.players[winner_index].chips += self.pot;
            self.pot = 0;
        }

        self.state = GameState::Finished;
    }

    pub fn get_game_state(&self) -> serde_json::Value {
        serde_json::json!({
            "game_id": self.id,
            "state": self.state,
            "pot": self.pot,
            "current_bet": self.current_bet,
            "current_player": if self.players.len() > 0 { 
                Some(&self.players[self.current_player_index].id) 
            } else { 
                None 
            },
            "community_cards": self.community_cards,
            "players": self.players.iter().map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "chips": p.chips,
                "current_bet": p.current_bet,
                "is_folded": p.is_folded,
                "is_all_in": p.is_all_in,
                "hand": if matches!(self.state, GameState::Showdown | GameState::Finished) {
                    p.hand.clone()
                } else {
                    vec![]
                }
            })).collect::<Vec<_>>()
        })
    }
}
