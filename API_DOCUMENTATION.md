# API de Poker - Documentação

## Funcionalidades Implementadas

### 1. Distribuição de Cartas
- Cada jogador recebe 2 cartas na mão (hole cards)
- Community cards são distribuídas em 3 fases: Flop (3 cartas), Turn (1 carta), River (1 carta)
- O deck é embaralhado aleatoriamente a cada nova mão

### 2. Avaliação de Mãos
Sistema completo de avaliação de mãos de poker implementado:
- Royal Flush
- Straight Flush
- Four of a Kind
- Full House
- Flush
- Straight
- Three of a Kind
- Two Pair
- One Pair
- High Card

### 3. Determinação do Vencedor
- Avalia automaticamente as melhores mãos de 5 cartas de cada jogador
- Suporta empates e distribui o pot igualmente entre vencedores
- Considera todas as 7 cartas disponíveis (2 da mão + 5 comunitárias)

## Endpoints da API

### Criar Sala
```http
POST /room
Content-Type: application/json

{
  "creator_name": "João",
  "max_players": 6
}
```

**Resposta:**
```json
{
  "room_id": "abc12345",
  "player_id": "uuid-do-jogador"
}
```

### Entrar na Sala
```http
POST /room/{room_id}/join
Content-Type: application/json

{
  "player_name": "Maria"
}
```

**Resposta:**
```json
{
  "success": true,
  "message": "Entrou na sala com sucesso",
  "player_id": "uuid-do-jogador"
}
```

### Iniciar Jogo
```http
POST /room/{room_id}/start
```

**Resposta:**
```json
{
  "success": true,
  "message": "Jogo iniciado",
  "game_state": {
    "game_id": "uuid-do-jogo",
    "state": "PreFlop",
    "pot": 15,
    "current_bet": 10,
    "current_player": "uuid-do-jogador-atual",
    "community_cards": [],
    "players": [
      {
        "id": "uuid-jogador1",
        "name": "João",
        "chips": 990,
        "current_bet": 10,
        "is_folded": false,
        "is_all_in": false,
        "hand": [
          {"suit": "Hearts", "rank": "Ace"},
          {"suit": "Spades", "rank": "King"}
        ]
      }
    ]
  }
}
```

### Obter Resultado da Mão
```http
GET /room/{room_id}/result
```

**Resposta (quando jogo terminou):**
```json
{
  "success": true,
  "result": {
    "type": "showdown",
    "winners": [
      {
        "id": "uuid-jogador",
        "name": "João",
        "hand": [...],
        "best_hand": [...],
        "hand_rank": "OnePair"
      }
    ],
    "all_hands": [...]
  }
}
```

### Iniciar Nova Mão
```http
POST /room/{room_id}/next
```

**Resposta:**
```json
{
  "success": true,
  "message": "Nova mão iniciada",
  "game_state": {...}
}
```

### WebSocket para Ações do Jogo
```http
GET /room/{room_id}/ws
```

**Mensagens enviadas via WebSocket:**

Para fazer uma ação (fold, check, call, raise, all-in):
```json
{
  "message_type": "game_action",
  "data": {
    "player_id": "uuid-do-jogador",
    "action": "Call"
  }
}
```

Para raise:
```json
{
  "message_type": "game_action",
  "data": {
    "player_id": "uuid-do-jogador",
    "action": {"Raise": 20}
  }
}
```

**Mensagens recebidas via WebSocket:**

Quando o jogo termina, você receberá uma mensagem `round_finished` com o resultado detalhado:

Para showdown (múltiplos jogadores):
```json
{
  "type": "round_finished",
  "data": {
    "type": "showdown",
    "pot_won": 120,
    "winners": [
      {
        "id": "uuid-jogador",
        "name": "João",
        "hand": [
          {"suit": "Hearts", "rank": "Ace"},
          {"suit": "Spades", "rank": "King"}
        ],
        "best_hand": [...], 
        "hand_rank": "OnePair"
      }
    ],
    "all_hands": [...]
  }
}
```

Para vencedor único:
```json
{
  "type": "round_finished",
  "data": {
    "type": "single_winner",
    "pot_won": 120,
    "winner": {
      "id": "uuid-jogador",
      "name": "João",
      "hand": [...]
    }
  }
}
```

## Estados do Jogo

1. **Waiting** - Aguardando jogadores
2. **PreFlop** - Cartas distribuídas, rodada de apostas inicial
3. **Flop** - 3 cartas comunitárias reveladas
4. **Turn** - 4ª carta comunitária revelada
5. **River** - 5ª carta comunitária revelada
6. **Showdown** - Revelação das mãos
7. **Finished** - Mão terminada, vencedor determinado

## Ações dos Jogadores

- **Fold** - Desistir da mão
- **Check** - Passar (quando não há aposta)
- **Call** - Igualar a aposta atual
- **Raise(amount)** - Aumentar a aposta
- **AllIn** - Apostar todas as fichas

## Características Importantes

1. **Cartas Visíveis**: Por padrão, as cartas de todos os jogadores são visíveis na resposta da API (como solicitado). Você pode controlar a visibilidade no front-end.

2. **Avaliação Automática**: O sistema automaticamente avalia e determina o vencedor ao final de cada mão.

3. **Dealer Button**: O dealer avança automaticamente a cada nova mão.

4. **Blinds**: Small blind e big blind são postados automaticamente no início de cada mão.

5. **Gestão de Pot**: O pot é automaticamente distribuído aos vencedores, considerando empates.

## Exemplo de Fluxo Completo

1. Criar sala (`POST /room`)
2. Jogadores entram (`POST /room/{id}/join`)
3. Iniciar jogo (`POST /room/{id}/start`)
4. Jogadores fazem ações via WebSocket
5. Jogo progride automaticamente pelas fases
6. Vencedor é determinado automaticamente
7. Obter resultado (`GET /room/{id}/result`)
8. Iniciar nova mão (`POST /room/{id}/next`)

## Rodando o Servidor

```bash
cargo run
```

O servidor estará disponível em `http://0.0.0.0:3000`
