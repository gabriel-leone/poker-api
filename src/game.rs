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
    }    pub fn start_round(&mut self) {
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

        // Set current player (left of big blind - que é dealer + 3 posições)
        self.current_player_index = self.get_first_active_player_after_big_blind();
    }

    fn get_first_active_player_after_big_blind(&self) -> usize {
        // Big blind está na posição dealer + 2
        // Primeiro jogador após big blind está na posição dealer + 3
        let start_index = (self.dealer_index + 3) % self.players.len();
        
        // Procurar o primeiro jogador ativo a partir desta posição
        for i in 0..self.players.len() {
            let index = (start_index + i) % self.players.len();
            let player = &self.players[index];
            if !player.is_folded && !player.is_all_in {
                return index;
            }
        }
        
        // Fallback - retorna o índice calculado se nenhum jogador ativo for encontrado
        start_index
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
    }    pub fn process_action(&mut self, player_id: &str, action: PlayerAction) -> Result<(), String> {
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
            }            PlayerAction::Check => {
                if self.current_bet > self.players[self.current_player_index].current_bet {
                    return Err("Não é possível dar check, há uma aposta a ser igualada".to_string());
                }
                // Registrar que este jogador fez uma ação (check) nesta rodada
                let current_total_bet = *self.round_bets.get(&self.players[self.current_player_index].id).unwrap_or(&0);
                self.round_bets.insert(self.players[self.current_player_index].id.clone(), current_total_bet);
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
            }        }

        self.next_player();
        self.check_round_completion();

        Ok(())
    }    fn next_player(&mut self) {
        let starting_index = self.current_player_index;
        
        loop {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
            let player = &self.players[self.current_player_index];
            
            // Se encontramos um jogador ativo, pare aqui
            if !player.is_folded && !player.is_all_in {
                break;
            }
            
            // Se voltamos ao índice inicial, significa que não há mais jogadores ativos
            // para continuar a rodada de apostas
            if self.current_player_index == starting_index {
                break;
            }
        }
    }fn check_round_completion(&mut self) {
        let active_players: Vec<_> = self.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded && !p.is_all_in)
            .collect();

        // Se há apenas um jogador ativo ou menos, a rodada termina
        if active_players.len() <= 1 {
            self.advance_game_state();
            return;
        }

        // Verificar se todos os jogadores ativos fizeram a mesma aposta
        let current_bet = self.current_bet;
        let all_bets_equal = active_players.iter()
            .all(|(_, p)| p.current_bet == current_bet);

        // Verificar se todos os jogadores ativos já tiveram sua vez na rodada atual
        // Isso garante que não avançamos prematuramente quando um jogador ainda não teve sua chance
        let all_players_had_turn = self.has_betting_round_completed(&active_players);

        if all_bets_equal && all_players_had_turn {
            self.advance_game_state();
        }
    }    fn has_betting_round_completed(&self, active_players: &[(usize, &Player)]) -> bool {
        // Se não há aposta atual, verificar se todos os jogadores ativos tiveram sua vez
        if self.current_bet == 0 {
            // Para que a rodada termine sem apostas, todos devem ter pelo menos uma entrada em round_bets
            // (indicando que fizeram pelo menos um Check)
            return active_players.iter().all(|(_, player)| {
                self.round_bets.contains_key(&player.id)
            });
        }
        
        // Se há apostas, verificar se todos os jogadores ativos fizeram pelo menos uma ação
        // na rodada atual (ou seja, todos têm uma entrada em round_bets ou igualaram a aposta)
        active_players.iter().all(|(_, player)| {
            player.current_bet == self.current_bet || 
            self.round_bets.contains_key(&player.id)
        })
    }    fn advance_game_state(&mut self) {
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
                return; // Não redefinir current_player_index para showdown
            }
            _ => {}
        }

        // Reset current player para o primeiro jogador ativo à esquerda do dealer
        self.current_player_index = self.get_first_active_player_after_dealer();
    }

    fn get_first_active_player_after_dealer(&self) -> usize {
        for i in 1..=self.players.len() {
            let index = (self.dealer_index + i) % self.players.len();
            let player = &self.players[index];
            if !player.is_folded && !player.is_all_in {
                return index;
            }
        }
        // Fallback - retorna o dealer se nenhum jogador ativo for encontrado
        self.dealer_index
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_players() -> Vec<Player> {
        vec![
            Player {
                id: "player1".to_string(),
                name: "Alice".to_string(),
                chips: 1000,
                hand: Vec::new(),
                current_bet: 0,
                is_folded: false,
                is_all_in: false,
            },
            Player {
                id: "player2".to_string(),
                name: "Bob".to_string(),
                chips: 1000,
                hand: Vec::new(),
                current_bet: 0,
                is_folded: false,
                is_all_in: false,
            },
            Player {
                id: "player3".to_string(),
                name: "Charlie".to_string(),
                chips: 1000,
                hand: Vec::new(),
                current_bet: 0,
                is_folded: false,
                is_all_in: false,
            },
        ]
    }

    #[test]
    fn test_game_creation() {
        let players = create_test_players();
        let game = Game::new(players.clone());
        
        assert_eq!(game.players.len(), 3);
        assert_eq!(game.state, GameState::PreFlop);
        assert_eq!(game.pot, 0);
        assert_eq!(game.current_bet, 0);
        assert_eq!(game.current_player_index, 0);
        assert_eq!(game.dealer_index, 0);
    }

    #[test]
    fn test_game_start_round() {
        let players = create_test_players();
        let mut game = Game::new(players);
        
        game.start_round();
        
        // Verificar que os blinds foram postados
        assert!(game.pot > 0);
        assert!(game.current_bet > 0);
        
        // Verificar que cada jogador recebeu 2 cartas
        for player in &game.players {
            assert_eq!(player.hand.len(), 2);
        }
        
        // Verificar que o estado é PreFlop
        assert_eq!(game.state, GameState::PreFlop);
    }

    #[test]
    fn test_turn_validation_rejects_wrong_player() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        println!("Current player index: {}", game.current_player_index);
        println!("Current player ID: {}", game.players[game.current_player_index].id);
        
        // Tentar fazer uma ação com um jogador que não é o atual
        let wrong_player_id = if game.players[game.current_player_index].id == "player1" {
            "player2"
        } else {
            "player1"
        };
        
        let result = game.process_action(wrong_player_id, PlayerAction::Check);
        
        // Deve retornar erro
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Não é sua vez de jogar");
    }

    #[test]
    fn test_turn_validation_accepts_correct_player() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let current_player_id = game.players[game.current_player_index].id.clone();
        
        // Se há uma aposta atual (big blind), fazer call em vez de check
        let action = if game.current_bet > 0 {
            PlayerAction::Call
        } else {
            PlayerAction::Check
        };
        
        let result = game.process_action(&current_player_id, action);
        
        // Deve ser aceito
        assert!(result.is_ok());
    }

    #[test]
    fn test_game_state_does_not_advance_on_invalid_action() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        let initial_community_cards = game.community_cards.len();
        let initial_current_player = game.current_player_index;
        
        // Tentar fazer uma ação com jogador errado
        let wrong_player_id = if game.players[game.current_player_index].id == "player1" {
            "player2"
        } else {
            "player1"
        };
        
        let result = game.process_action(wrong_player_id, PlayerAction::Check);
        
        // Verificar que a ação foi rejeitada
        assert!(result.is_err());
        
        // Verificar que o estado do jogo NÃO mudou
        assert_eq!(game.state, initial_state);
        assert_eq!(game.community_cards.len(), initial_community_cards);
        assert_eq!(game.current_player_index, initial_current_player);
    }

    #[test]
    fn test_multiple_invalid_actions_do_not_advance_game() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        let initial_community_cards = game.community_cards.len();
        let initial_current_player = game.current_player_index;
        
        // Fazer várias ações inválidas
        for _ in 0..5 {
            let wrong_player_id = if game.players[game.current_player_index].id == "player1" {
                "player2"
            } else {
                "player1"
            };
            
            let result = game.process_action(wrong_player_id, PlayerAction::Check);
            assert!(result.is_err());
        }
        
        // Verificar que o estado do jogo NÃO mudou após múltiplas ações inválidas
        assert_eq!(game.state, initial_state);
        assert_eq!(game.community_cards.len(), initial_community_cards);
        assert_eq!(game.current_player_index, initial_current_player);
    }

    #[test]
    fn test_game_progression_after_valid_actions() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        assert_eq!(initial_state, GameState::PreFlop);
        
        // Todos os jogadores fazem call ou check em sequência
        let num_players = game.players.len();
        for i in 0..num_players {
            let current_player_id = game.players[game.current_player_index].id.clone();
            
            // Determinar a ação apropriada
            let action = if game.current_bet > game.players[game.current_player_index].current_bet {
                PlayerAction::Call
            } else {
                PlayerAction::Check
            };
            
            println!("Player {} ({}) making action: {:?}", 
                     i, current_player_id, action);
            
            let result = game.process_action(&current_player_id, action);
            assert!(result.is_ok(), "Action failed for player {}: {:?}", current_player_id, result);
        }
        
        // Após todos fazerem suas ações, o jogo deve avançar para o Flop
        assert_eq!(game.state, GameState::Flop);
        assert_eq!(game.community_cards.len(), 3); // Flop tem 3 cartas
    }

    #[test]
    fn test_interleaved_invalid_and_valid_actions() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        // Fazer uma ação inválida
        let wrong_player_id = if game.players[game.current_player_index].id == "player1" {
            "player2"
        } else {
            "player1"
        };
        
        let invalid_result = game.process_action(wrong_player_id, PlayerAction::Check);
        assert!(invalid_result.is_err());
        
        let state_after_invalid = game.state.clone();
        let cards_after_invalid = game.community_cards.len();
        let player_after_invalid = game.current_player_index;
        
        // Fazer uma ação válida
        let current_player_id = game.players[game.current_player_index].id.clone();
        let action = if game.current_bet > game.players[game.current_player_index].current_bet {
            PlayerAction::Call
        } else {
            PlayerAction::Check
        };
        
        let valid_result = game.process_action(&current_player_id, action);
        assert!(valid_result.is_ok());
        
        // Verificar que apenas a ação válida teve efeito
        // (O jogador atual deve ter mudado após a ação válida)
        assert_ne!(game.current_player_index, player_after_invalid);
    }    #[test] 
    fn test_betting_round_completion_logic() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        assert_eq!(initial_state, GameState::PreFlop);
        
        // Fazer algumas ações inválidas no meio do jogo
        let initial_current_player = game.current_player_index;
        
        // Primeira ação inválida
        let wrong_player_id = if game.players[game.current_player_index].id == "player1" {
            "player2"
        } else {
            "player1"
        };
        
        let invalid_result = game.process_action(wrong_player_id, PlayerAction::Call);
        assert!(invalid_result.is_err());
        
        // Verificar que o estado não mudou após ação inválida
        assert_eq!(game.state, initial_state);
        assert_eq!(game.current_player_index, initial_current_player);
        
        // Agora fazer uma ação válida
        let current_player_id = game.players[game.current_player_index].id.clone();
        let action = if game.current_bet > game.players[game.current_player_index].current_bet {
            PlayerAction::Call
        } else {
            PlayerAction::Check
        };
        
        let valid_result = game.process_action(&current_player_id, action);
        assert!(valid_result.is_ok());
        
        // Após uma ação válida, o jogador atual deve ter mudado
        assert_ne!(game.current_player_index, initial_current_player);
        
        // Mas ainda deve estar no PreFlop (só 1 jogador fez ação)
        assert_eq!(game.state, GameState::PreFlop);
    }

    #[test]
    fn test_rapid_invalid_actions_stress_test() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        let initial_community_cards = game.community_cards.len();
        let initial_current_player = game.current_player_index;
        
        // Simular 100 ações inválidas rápidas
        for i in 0..100 {
            let wrong_player_id = format!("wrong_player_{}", i);
            let result = game.process_action(&wrong_player_id, PlayerAction::Check);
            assert!(result.is_err());
        }
        
        // O jogo deve permanecer no mesmo estado
        assert_eq!(game.state, initial_state);
        assert_eq!(game.community_cards.len(), initial_community_cards);
        assert_eq!(game.current_player_index, initial_current_player);
        
        // Uma ação válida ainda deve funcionar
        let current_player_id = game.players[game.current_player_index].id.clone();
        let action = if game.current_bet > game.players[game.current_player_index].current_bet {
            PlayerAction::Call
        } else {
            PlayerAction::Check
        };
        
        let valid_result = game.process_action(&current_player_id, action);
        assert!(valid_result.is_ok());
    }

    #[test]
    fn test_has_betting_round_completed_logic() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        // No início do PreFlop, há big blind, então current_bet > 0
        assert!(game.current_bet > 0);
        
        // Resetar para simular início de uma nova fase (Flop)
        game.state = GameState::Flop;
        game.current_bet = 0;
        for player in &mut game.players {
            player.current_bet = 0;
        }
        game.round_bets.clear();
        
        // Agora current_bet == 0, mas nem todos tiveram sua vez
        let active_players: Vec<_> = game.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded && !p.is_all_in)
            .collect();
        
        // has_betting_round_completed deve retornar true quando current_bet == 0
        // Isso pode ser o bug!
        let round_completed = game.has_betting_round_completed(&active_players);
        println!("Round completed when current_bet=0: {}", round_completed);
        
        // Se retornar true quando current_bet=0, isso pode causar advance prematuro
        if round_completed {
            println!("WARNING: has_betting_round_completed returns true when current_bet=0");
            println!("This could cause premature game state advancement!");
        }
    }

    #[test]
    fn test_invalid_actions_should_not_advance_game_state() {
        let players = create_test_players();
        let mut game = Game::new(players);
        game.start_round();
        
        let initial_state = game.state.clone();
        let initial_community_cards = game.community_cards.len();
        let initial_current_player = game.current_player_index;
        
        println!("Initial state: {:?}", initial_state);
        println!("Initial current player: {}", game.players[initial_current_player].id);
        
        // Fazer 10 ações inválidas consecutivas
        for i in 0..10 {
            let wrong_player_id = format!("invalid_player_{}", i);
            let result = game.process_action(&wrong_player_id, PlayerAction::Check);
            
            // Verificar que foi rejeitada
            assert!(result.is_err());
            
            // Verificar que NADA mudou
            assert_eq!(game.state, initial_state, "Game state changed after invalid action {}", i);
            assert_eq!(game.community_cards.len(), initial_community_cards, "Community cards changed after invalid action {}", i);
            assert_eq!(game.current_player_index, initial_current_player, "Current player changed after invalid action {}", i);
        }
        
        println!("After 10 invalid actions - state: {:?}", game.state);
        println!("Current player still: {}", game.players[game.current_player_index].id);
        
        // O jogo ainda deve estar exatamente no mesmo estado
        assert_eq!(game.state, GameState::PreFlop);
        assert_eq!(game.community_cards.len(), 0);
        
        // Uma ação válida ainda deve funcionar normalmente
        let current_player_id = game.players[game.current_player_index].id.clone();
        let action = if game.current_bet > 0 {
            PlayerAction::Call
        } else {
            PlayerAction::Check
        };
        
        let valid_result = game.process_action(&current_player_id, action);
        assert!(valid_result.is_ok(), "Valid action should work after invalid ones");
        
        // Após UMA ação válida, ainda deve estar no PreFlop
        assert_eq!(game.state, GameState::PreFlop, "Should still be in PreFlop after only one valid action");
    }
}
