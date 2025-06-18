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

        // Recriar e embaralhar o deck
        self.deck = Self::create_deck();
        let mut rng = thread_rng();
        self.deck.shuffle(&mut rng);

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
        self.round_bets.insert(self.players[big_blind_index].id.clone(), big_blind_amount);    }    pub fn process_action(&mut self, player_id: &str, action: PlayerAction) -> Result<Option<serde_json::Value>, String> {
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
        let round_result = self.check_round_completion();

        Ok(round_result)
    }fn next_player(&mut self) {
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
    }    fn check_round_completion(&mut self) -> Option<serde_json::Value> {
        let active_players: Vec<_> = self.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded && !p.is_all_in)
            .collect();

        // Se há apenas um jogador ativo ou menos, a rodada termina
        if active_players.len() <= 1 {
            return self.advance_game_state();
        }

        // Verificar se todos os jogadores ativos fizeram a mesma aposta
        let current_bet = self.current_bet;
        let all_bets_equal = active_players.iter()
            .all(|(_, p)| p.current_bet == current_bet);

        // Verificar se todos os jogadores ativos já tiveram sua vez na rodada atual
        // Isso garante que não avançamos prematuramente quando um jogador ainda não teve sua chance
        let all_players_had_turn = self.has_betting_round_completed(&active_players);

        if all_bets_equal && all_players_had_turn {
            return self.advance_game_state();
        }

        None
    }fn has_betting_round_completed(&self, active_players: &[(usize, &Player)]) -> bool {
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
    }    fn advance_game_state(&mut self) -> Option<serde_json::Value> {
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
                return self.determine_winner(); // Retornar o resultado
            }
            _ => {}
        }

        // Reset current player para o primeiro jogador ativo à esquerda do dealer
        self.current_player_index = self.get_first_active_player_after_dealer();
        None
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
    }    fn determine_winner(&mut self) -> Option<serde_json::Value> {
        let active_players: Vec<_> = self.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded)
            .collect();

        let pot_amount = self.pot; // Capturar o valor do pot antes de limpar        // Se só há um jogador ativo, ele ganha
        if active_players.len() == 1 {
            let (winner_index, _) = active_players[0];
            let winner_id = self.players[winner_index].id.clone();
            let winner_name = self.players[winner_index].name.clone();
            let winner_hand = self.players[winner_index].hand.clone();
            
            self.players[winner_index].chips += self.pot;
            let result = serde_json::json!({
                "type": "single_winner",
                "winner": {
                    "id": winner_id,
                    "name": winner_name,
                    "hand": winner_hand
                },
                "pot_won": pot_amount
            });
            self.pot = 0;
            self.state = GameState::Finished;
            return Some(result);
        }

        // Avaliar todas as mãos dos jogadores ativos
        let mut evaluations: Vec<(usize, HandEvaluation)> = Vec::new();
        
        for (index, player) in &active_players {
            let mut all_cards = player.hand.clone();
            all_cards.extend(self.community_cards.clone());
            let evaluation = self.evaluate_hand(all_cards);
            evaluations.push((*index, evaluation));
        }

        // Ordenar por força da mão (melhor mão primeiro)
        evaluations.sort_by(|a, b| b.1.cmp(&a.1));

        // Determinar vencedores (pode haver empate)
        let best_hand = &evaluations[0].1;
        let mut winners = Vec::new();
        
        for (index, eval) in &evaluations {
            if eval == best_hand {
                winners.push(*index);
            } else {
                break;
            }
        }

        // Distribuir o pot entre os vencedores
        let pot_per_winner = self.pot / winners.len() as u32;
        let remainder = self.pot % winners.len() as u32;

        for (i, &winner_index) in winners.iter().enumerate() {
            let mut winnings = pot_per_winner;
            if i < remainder as usize {
                winnings += 1; // Distribuir o resto
            }
            self.players[winner_index].chips += winnings;
        }

        // Criar resultado detalhado
        let result = serde_json::json!({
            "type": "showdown",
            "pot_won": pot_amount,
            "winners": winners.iter().map(|&index| {
                let player = &self.players[index];
                let eval = evaluations.iter().find(|(i, _)| *i == index).unwrap().1.clone();
                serde_json::json!({
                    "id": player.id,
                    "name": player.name,
                    "hand": player.hand,
                    "best_hand": eval.cards,
                    "hand_rank": eval.rank
                })
            }).collect::<Vec<_>>(),
            "all_hands": evaluations.iter().map(|(index, eval)| {
                let player = &self.players[*index];
                serde_json::json!({
                    "id": player.id,
                    "name": player.name,
                    "hand": player.hand,
                    "best_hand": eval.cards,
                    "hand_rank": eval.rank
                })
            }).collect::<Vec<_>>()
        });

        self.pot = 0;
        self.state = GameState::Finished;
        Some(result)
    }

    fn evaluate_hand(&self, cards: Vec<Card>) -> HandEvaluation {
        // Obter todas as combinações de 5 cartas das 7 disponíveis
        let combinations = self.get_five_card_combinations(cards);
        
        // Avaliar cada combinação e retornar a melhor
        let mut best_evaluation = self.evaluate_five_cards(&combinations[0]);
        
        for combination in combinations.iter().skip(1) {
            let evaluation = self.evaluate_five_cards(combination);
            if evaluation > best_evaluation {
                best_evaluation = evaluation;
            }
        }
        
        best_evaluation
    }

    fn get_five_card_combinations(&self, cards: Vec<Card>) -> Vec<Vec<Card>> {
        let mut combinations = Vec::new();
        let n = cards.len();
        
        // Gerar todas as combinações de 5 cartas
        for i in 0..n {
            for j in (i+1)..n {
                for k in (j+1)..n {
                    for l in (k+1)..n {
                        for m in (l+1)..n {
                            combinations.push(vec![
                                cards[i].clone(),
                                cards[j].clone(),
                                cards[k].clone(),
                                cards[l].clone(),
                                cards[m].clone(),
                            ]);
                        }
                    }
                }
            }
        }
        
        combinations
    }

    fn evaluate_five_cards(&self, cards: &[Card]) -> HandEvaluation {
        let mut sorted_cards = cards.to_vec();
        sorted_cards.sort_by(|a, b| (b.rank as u8).cmp(&(a.rank as u8)));
        
        let ranks: Vec<u8> = sorted_cards.iter().map(|c| c.rank as u8).collect();
        let suits: Vec<Suit> = sorted_cards.iter().map(|c| c.suit).collect();
        
        let is_flush = suits.iter().all(|&s| s == suits[0]);
        let is_straight = self.is_straight(&ranks);
        
        // Royal Flush
        if is_flush && is_straight && ranks[0] == 14 {
            return HandEvaluation {
                rank: HandRank::RoyalFlush,
                kickers: vec![14],
                cards: sorted_cards,
            };
        }
        
        // Straight Flush
        if is_flush && is_straight {
            return HandEvaluation {
                rank: HandRank::StraightFlush,
                kickers: vec![ranks[0]],
                cards: sorted_cards,
            };
        }
        
        // Contar frequências dos ranks
        let mut rank_counts: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
        for &rank in &ranks {
            *rank_counts.entry(rank).or_insert(0) += 1;
        }
        
        let mut counts: Vec<(u8, u8)> = rank_counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
        
        // Four of a Kind
        if counts[0].1 == 4 {
            return HandEvaluation {
                rank: HandRank::FourOfAKind,
                kickers: vec![counts[0].0, counts[1].0],
                cards: sorted_cards,
            };
        }
        
        // Full House
        if counts[0].1 == 3 && counts[1].1 == 2 {
            return HandEvaluation {
                rank: HandRank::FullHouse,
                kickers: vec![counts[0].0, counts[1].0],
                cards: sorted_cards,
            };
        }
        
        // Flush
        if is_flush {
            return HandEvaluation {
                rank: HandRank::Flush,
                kickers: ranks,
                cards: sorted_cards,
            };
        }
        
        // Straight
        if is_straight {
            return HandEvaluation {
                rank: HandRank::Straight,
                kickers: vec![ranks[0]],
                cards: sorted_cards,
            };
        }
        
        // Three of a Kind
        if counts[0].1 == 3 {
            return HandEvaluation {
                rank: HandRank::ThreeOfAKind,
                kickers: vec![counts[0].0, counts[1].0, counts[2].0],
                cards: sorted_cards,
            };
        }
        
        // Two Pair
        if counts[0].1 == 2 && counts[1].1 == 2 {
            return HandEvaluation {
                rank: HandRank::TwoPair,
                kickers: vec![counts[0].0, counts[1].0, counts[2].0],
                cards: sorted_cards,
            };
        }
        
        // One Pair
        if counts[0].1 == 2 {
            return HandEvaluation {
                rank: HandRank::OnePair,
                kickers: vec![counts[0].0, counts[1].0, counts[2].0, counts[3].0],
                cards: sorted_cards,
            };
        }
        
        // High Card
        HandEvaluation {
            rank: HandRank::HighCard,
            kickers: ranks,
            cards: sorted_cards,
        }
    }

    fn is_straight(&self, ranks: &[u8]) -> bool {
        // Verificar sequência normal
        let mut consecutive = true;
        for i in 1..ranks.len() {
            if ranks[i-1] - ranks[i] != 1 {
                consecutive = false;
                break;
            }
        }
        
        if consecutive {
            return true;
        }
        
        // Verificar A-2-3-4-5 (wheel)
        if ranks == [14, 5, 4, 3, 2] {
            return true;
        }
        
        false
    }    pub fn get_game_state(&self) -> serde_json::Value {
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
                "hand": p.hand // Sempre mostrar as cartas dos jogadores
            })).collect::<Vec<_>>()
        })
    }

    pub fn next_hand(&mut self) {
        // Avançar o dealer para o próximo jogador
        self.dealer_index = (self.dealer_index + 1) % self.players.len();
        
        // Começar nova rodada
        self.start_round();
    }

    pub fn get_hand_result(&self) -> Option<serde_json::Value> {
        if !matches!(self.state, GameState::Finished) {
            return None;
        }

        let active_players: Vec<_> = self.players.iter()
            .enumerate()
            .filter(|(_, p)| !p.is_folded)
            .collect();

        if active_players.len() <= 1 {
            let winner = &active_players[0].1;
            return Some(serde_json::json!({
                "type": "single_winner",
                "winner": {
                    "id": winner.id,
                    "name": winner.name,
                    "hand": winner.hand
                },
                "pot_won": 0 // Pot já foi distribuído
            }));
        }

        // Avaliar todas as mãos para mostrar o resultado
        let mut evaluations: Vec<(usize, HandEvaluation)> = Vec::new();
        
        for (index, player) in &active_players {
            let mut all_cards = player.hand.clone();
            all_cards.extend(self.community_cards.clone());
            let evaluation = self.evaluate_hand(all_cards);
            evaluations.push((*index, evaluation));
        }

        // Ordenar por força da mão
        evaluations.sort_by(|a, b| b.1.cmp(&a.1));

        // Determinar vencedores
        let best_hand = &evaluations[0].1;
        let mut winners = Vec::new();
        
        for (index, eval) in &evaluations {
            if eval == best_hand {
                winners.push((*index, eval.clone()));
            } else {
                break;
            }
        }

        Some(serde_json::json!({
            "type": "showdown",
            "winners": winners.iter().map(|(index, eval)| {
                let player = &self.players[*index];
                serde_json::json!({
                    "id": player.id,
                    "name": player.name,
                    "hand": player.hand,
                    "best_hand": eval.cards,
                    "hand_rank": eval.rank
                })
            }).collect::<Vec<_>>(),
            "all_hands": evaluations.iter().map(|(index, eval)| {
                let player = &self.players[*index];
                serde_json::json!({
                    "id": player.id,
                    "name": player.name,
                    "hand": player.hand,
                    "best_hand": eval.cards,
                    "hand_rank": eval.rank
                })
            }).collect::<Vec<_>>()
        }))
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
        // Verificar que não retornou resultado de fim de jogo ainda
        assert!(result.unwrap().is_none());
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

    #[test]
    fn test_hand_evaluation_royal_flush() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Hearts, rank: Rank::Ten },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::RoyalFlush);
    }

    #[test]
    fn test_hand_evaluation_straight_flush() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Eight },
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::StraightFlush);
    }

    #[test]
    fn test_hand_evaluation_four_of_a_kind() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::Ace },
            Card { suit: Suit::Spades, rank: Rank::Ace },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::FourOfAKind);
    }

    #[test]
    fn test_hand_evaluation_full_house() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::Ace },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::FullHouse);
    }

    #[test]
    fn test_hand_evaluation_flush() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::Flush);
    }

    #[test]
    fn test_hand_evaluation_straight() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Queen },
            Card { suit: Suit::Spades, rank: Rank::Jack },
            Card { suit: Suit::Hearts, rank: Rank::Ten },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::Straight);
    }

    #[test]
    fn test_hand_evaluation_wheel_straight() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Spades, rank: Rank::Four },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Queen },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::Straight);
    }

    #[test]
    fn test_hand_evaluation_three_of_a_kind() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::Ace },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::ThreeOfAKind);
    }

    #[test]
    fn test_hand_evaluation_two_pair() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::TwoPair);
    }

    #[test]
    fn test_hand_evaluation_one_pair() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::OnePair);
    }

    #[test]
    fn test_hand_evaluation_high_card() {
        let players = create_test_players();
        let game = Game::new(players);
        
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Jack },
            Card { suit: Suit::Spades, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
        ];
        
        let evaluation = game.evaluate_hand(cards);
        assert_eq!(evaluation.rank, HandRank::HighCard);
    }

    #[test]
    fn test_complete_game_with_winner_determination() {
        let players = create_test_players();
        let mut game = Game::new(players);
        
        // Começar a rodada
        game.start_round();
        
        // Verificar que os jogadores receberam cartas
        for player in &game.players {
            assert_eq!(player.hand.len(), 2);
        }
        
        // Simular que todos os jogadores fazem call/check para avançar para o flop
        let num_players = game.players.len();
        for _ in 0..num_players {
            let current_player_id = game.players[game.current_player_index].id.clone();
            let action = if game.current_bet > game.players[game.current_player_index].current_bet {
                PlayerAction::Call
            } else {
                PlayerAction::Check
            };
            let _ = game.process_action(&current_player_id, action);
        }
        
        // Deve estar no flop agora
        assert_eq!(game.state, GameState::Flop);
        assert_eq!(game.community_cards.len(), 3);
        
        // Continuar até o river
        for _ in 0..num_players {
            let current_player_id = game.players[game.current_player_index].id.clone();
            let _ = game.process_action(&current_player_id, PlayerAction::Check);
        }
        
        assert_eq!(game.state, GameState::Turn);
        assert_eq!(game.community_cards.len(), 4);
        
        for _ in 0..num_players {
            let current_player_id = game.players[game.current_player_index].id.clone();
            let _ = game.process_action(&current_player_id, PlayerAction::Check);
        }
        
        assert_eq!(game.state, GameState::River);
        assert_eq!(game.community_cards.len(), 5);
        
        for _ in 0..num_players {
            let current_player_id = game.players[game.current_player_index].id.clone();
            let _ = game.process_action(&current_player_id, PlayerAction::Check);
        }
        
        // Deve ter terminado
        assert_eq!(game.state, GameState::Finished);
        
        // Verificar que um vencedor foi determinado (pot foi distribuído)
        assert_eq!(game.pot, 0);
        
        // Verificar que pelo menos um jogador ganhou fichas
        let total_chips: u32 = game.players.iter().map(|p| p.chips).sum();
        assert!(total_chips > 0);
    }
}
