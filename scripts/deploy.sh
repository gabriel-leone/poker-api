#!/bin/bash

# Script de deploy para Poker API
# Este script prepara e verifica se tudo está pronto para deploy

set -e

echo "🚀 Preparando deploy da Poker API..."

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Função para log colorido
log_info() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Verificar se estamos na branch main
current_branch=$(git branch --show-current)
if [ "$current_branch" != "main" ]; then
    log_warn "Você está na branch '$current_branch'. Para deploy em produção, use a branch 'main'."
    read -p "Continuar mesmo assim? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Verificar se há mudanças não commitadas
if ! git diff-index --quiet HEAD --; then
    log_error "Há mudanças não commitadas. Commit ou stash suas mudanças primeiro."
    exit 1
fi

log_info "Verificando formatação do código..."
if ! cargo fmt --check; then
    log_error "Código não está formatado. Execute: cargo fmt"
    exit 1
fi

log_info "Executando clippy (linter)..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    log_error "Clippy encontrou problemas. Corrija-os antes do deploy."
    exit 1
fi

log_info "Executando testes..."
if ! cargo test; then
    log_error "Testes falharam. Corrija-os antes do deploy."
    exit 1
fi

log_info "Verificando build de release..."
if ! cargo build --release; then
    log_error "Build de release falhou."
    exit 1
fi

# Verificar se cargo-audit está instalado
if ! command -v cargo-audit &> /dev/null; then
    log_warn "cargo-audit não encontrado. Instalando..."
    cargo install cargo-audit
fi

log_info "Executando auditoria de segurança..."
if ! cargo audit; then
    log_error "Auditoria de segurança falhou. Verifique as vulnerabilidades."
    exit 1
fi

log_info "Verificando se Docker build funcionaria..."
if command -v docker &> /dev/null; then
    if ! docker build -t poker-api:test .; then
        log_error "Docker build falhou."
        exit 1
    fi
    log_info "Docker build bem-sucedida!"
else
    log_warn "Docker não encontrado. Pulando teste de build Docker."
fi

echo
log_info "🎉 Todas as verificações passaram!"
echo
echo "📋 Próximos passos para deploy:"
echo "1. Faça push para a branch main: git push origin main"
echo "2. GitHub Actions executará automaticamente:"
echo "   - Testes"
echo "   - Linting"
echo "   - Security audit"
echo "   - Docker build"
echo "3. Se tudo passar, o Render fará deploy automaticamente"
echo
echo "🔗 Links úteis:"
echo "- GitHub Actions: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:\/]\([^.]*\).*/\1/')/actions"
echo "- Render Dashboard: https://dashboard.render.com/"
echo
log_info "Deploy preparado com sucesso! 🚀"
