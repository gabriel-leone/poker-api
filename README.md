# Poker API

Uma API REST e WebSocket para gerenciar jogos de pôquer multi-jogador em Rust.

## Características

- Criação e gerenciamento de salas de pôquer
- Sistema de entrada de jogadores via código da sala
- Comunicação em tempo real via WebSockets
- Lógica completa de jogo de pôquer Texas Hold'em
- Gerenciamento de fichas e apostas
- Suporte para múltiplas salas simultâneas

## Endpoints da API

### Criar Sala
```http
POST /room
Content-Type: application/json

{
    "creator_name": "Nome do Criador",
    "max_players": 6
}
```

Resposta:
```json
{
    "room_id": "abc12345"
}
```

### Entrar na Sala
```http
POST /room/{room_id}/join
Content-Type: application/json

{
    "player_name": "Nome do Jogador"
}
```

### Iniciar Jogo
```http
POST /room/{room_id}/start
```

### WebSocket
```
ws://localhost:3000/room/{room_id}/ws
```

## Mensagens WebSocket

### Conectar à sala
```json
{
    "message_type": "join",
    "data": {
        "player_id": "uuid-do-jogador"
    }
}
```

### Fazer uma jogada
```json
{
    "message_type": "game_action",
    "data": {
        "player_id": "uuid-do-jogador",
        "action": "Call" // ou "Fold", "Check", {"Raise": 50}, "AllIn"
    }
}
```

### Mensagens recebidas do servidor

#### Estado da sala
```json
{
    "type": "room_state",
    "data": {
        "room_id": "abc12345",
        "players": [...],
        "game": null
    }
}
```

#### Jogo iniciado
```json
{
    "type": "game_started",
    "data": {
        "game_id": "uuid",
        "state": "PreFlop",
        "pot": 15,
        "current_bet": 10,
        "current_player": "uuid-do-jogador",
        "community_cards": [],
        "players": [...]
    }
}
```

#### Atualização do jogo
```json
{
    "type": "game_update",
    "data": {
        // mesmo formato do game_started
    }
}
```

#### Erro
```json
{
    "type": "error",
    "data": {
        "message": "Não é sua vez de jogar"
    }
}
```

## Como executar

1. Instale o Rust: https://rustup.rs/
2. Clone/baixe este projeto
3. Execute:

```bash
cargo run
```

O servidor estará disponível em `http://localhost:3000`

## Fluxo do Jogo

1. **Criação da Sala**: Um jogador cria uma sala e recebe um código
2. **Entrada de Jogadores**: Outros jogadores usam o código para entrar
3. **Conexão WebSocket**: Cada jogador se conecta via WebSocket
4. **Início do Jogo**: O criador inicia o jogo (mínimo 2 jogadores)
5. **Gameplay**: Jogadores fazem suas jogadas em turnos
6. **Rodadas**: O jogo progride por Pre-flop, Flop, Turn, River e Showdown
7. **Nova Rodada**: Após cada mão, uma nova rodada inicia automaticamente

## Estados do Jogo

- **Waiting**: Aguardando jogadores
- **PreFlop**: Cartas individuais distribuídas, apostas iniciais
- **Flop**: 3 cartas comunitárias reveladas
- **Turn**: 4ª carta comunitária revelada
- **River**: 5ª carta comunitária revelada
- **Showdown**: Revelação das cartas e determinação do vencedor
- **Finished**: Rodada finalizada

## Ações dos Jogadores

- **Fold**: Desistir da mão
- **Check**: Passar a vez (sem apostar)
- **Call**: Igualar a aposta atual
- **Raise(amount)**: Aumentar a aposta
- **AllIn**: Apostar todas as fichas

## Exemplo de Uso com JavaScript

```javascript
// Criar sala
const response = await fetch('/room', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ creator_name: 'João', max_players: 4 })
});
const { room_id } = await response.json();

// Conectar WebSocket
const ws = new WebSocket(`ws://localhost:3000/room/${room_id}/ws`);

ws.onopen = () => {
    // Entrar na sala
    ws.send(JSON.stringify({
        message_type: 'join',
        data: { player_id: 'seu-player-id' }
    }));
};

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('Mensagem recebida:', message);
};

// Fazer uma jogada
function makeAction(action) {
    ws.send(JSON.stringify({
        message_type: 'game_action',
        data: {
            player_id: 'seu-player-id',
            action: action
        }
    }));
}

// Exemplos de jogadas
makeAction('Call');
makeAction('Fold');
makeAction({ Raise: 50 });
makeAction('AllIn');
```
