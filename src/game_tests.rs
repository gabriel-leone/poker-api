#[cfg(test)]
mod tests {
    use crate::game::*;
    use crate::models::*;

    #[test]
    fn test_card_creation() {
        let card = Card {
            suit: Suit::Hearts,
            rank: Rank::Ace,
        };
        assert_eq!(card.suit, Suit::Hearts);
        assert_eq!(card.rank, Rank::Ace);
    }

    #[test]
    fn test_deck_creation() {
        let deck = create_deck();
        assert_eq!(deck.len(), 52);
    }

    #[test]
    fn test_hand_evaluation_high_card() {
        let hand = vec![
            Card { suit: Suit::Hearts, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Five },
            Card { suit: Suit::Diamonds, rank: Rank::Seven },
            Card { suit: Suit::Spades, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
        ];
        let result = evaluate_hand(&hand);
        assert_eq!(result.hand_type, HandType::HighCard);
    }

    #[test]
    fn test_hand_evaluation_pair() {
        let hand = vec![
            Card { suit: Suit::Hearts, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Two },
            Card { suit: Suit::Diamonds, rank: Rank::Seven },
            Card { suit: Suit::Spades, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
        ];
        let result = evaluate_hand(&hand);
        assert_eq!(result.hand_type, HandType::Pair);
    }

    #[test]
    fn test_best_hand_from_seven_cards() {
        let cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Clubs, rank: Rank::Ten },
            Card { suit: Suit::Diamonds, rank: Rank::Two },
        ];
        let result = find_best_hand(&cards);
        // Deve encontrar uma sequência (straight)
        assert_eq!(result.hand_type, HandType::Straight);
    }

    #[test]
    fn test_compare_hands() {
        let hand1 = HandResult {
            hand_type: HandType::Pair,
            cards: vec![],
            kickers: vec![],
        };
        let hand2 = HandResult {
            hand_type: HandType::HighCard,
            cards: vec![],
            kickers: vec![],
        };
        assert!(hand1.hand_type as u8 > hand2.hand_type as u8);
    }
}