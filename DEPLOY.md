# Deploy Guide - Poker API

Este guia explica como fazer deploy da Poker API usando GitHub Actions e Render.

## 🚀 Configuração do Deploy

### 1. Configuração do GitHub

1. **Branch de Deploy**: Use a branch `main` para production
2. **GitHub Actions**: O workflow está configurado em `.github/workflows/ci-cd.yml`
3. **Proteção da Branch**: Configure branch protection rules para `main`

### 2. Configuração do Render

1. **Conecte seu repositório**: 
   - Vá em [render.com](https://render.com)
   - Conecte sua conta GitHub
   - Selecione este repositório

2. **Configure o serviço**:
   - Tipo: Web Service
   - Environment: Docker
   - Branch: `main`
   - Health Check Path: `/health`
   - Auto-Deploy: Yes

3. **Variáveis de ambiente**:
   ```
   PORT=10000 (Render define automaticamente)
   RUST_LOG=info
   RUST_BACKTRACE=1
   ```

### 3. Workflow de Deploy

O deploy acontece automaticamente quando:
1. ✅ **Testes passam** (unitários e integração)
2. ✅ **Lint/Formatting** está correto
3. ✅ **Security audit** passa
4. ✅ **Build Docker** é bem-sucedida
5. ✅ **Push na branch main**

## 🛠️ Comandos Úteis

### Desenvolvimento Local
```bash
# Rodar testes
cargo test

# Rodar com logs
RUST_LOG=debug cargo run

# Build para produção
cargo build --release

# Rodar Docker localmente
docker build -t poker-api .
docker run -p 3000:3000 poker-api
```

### Verificações antes do Deploy
```bash
# Formatação
cargo fmt --check

# Lint
cargo clippy -- -D warnings

# Testes
cargo test

# Security audit
cargo install cargo-audit
cargo audit
```

## 📦 Estrutura do Deploy

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   GitHub Repo   │────│  GitHub Actions │────│     Render      │
│                 │    │                 │    │                 │
│ • main branch   │    │ • Run tests     │    │ • Auto deploy   │
│ • Pull requests │    │ • Security scan │    │ • Health check  │
│ • Code review   │    │ • Build Docker  │    │ • Scale         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🔍 Monitoramento

- **Health Check**: `GET /health`
- **Logs**: Disponíveis no dashboard do Render
- **Status**: GitHub Actions mostra o status do pipeline

## 🔐 Segurança

- ✅ Cargo audit para vulnerabilidades
- ✅ Container sem privilégios
- ✅ Imagem mínima (Debian slim)
- ✅ Health checks configurados
- ✅ Variáveis de ambiente seguras

## 🚨 Troubleshooting

### Build falha
1. Verifique os logs no GitHub Actions
2. Teste localmente: `cargo build --release`
3. Verifique dependências no `Cargo.toml`

### Deploy falha no Render
1. Verifique os logs no dashboard do Render
2. Teste o Docker localmente
3. Verifique variáveis de ambiente

### Health check falha
1. Verifique se `/health` responde localmente
2. Verifique logs da aplicação
3. Verifique se a porta está correta

## 📝 Próximos Passos

1. **Monitoramento**: Adicionar Prometheus/Grafana
2. **Logging**: Estruturar logs com tracing
3. **Database**: Conectar PostgreSQL se necessário
4. **Cache**: Adicionar Redis para sessions
5. **CDN**: Configurar para assets estáticos
