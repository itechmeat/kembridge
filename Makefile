.PHONY: dev prod build clean logs test health

dev:
	docker-compose up --build

dev-detached:
	docker-compose up -d --build

prod:
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build

build:
	docker-compose build

clean:
	docker-compose down -v --rmi all

logs:
	docker-compose logs -f

test:
	docker-compose -f docker-compose.test.yml up --abort-on-container-exit

migrate:
	docker-compose exec backend sqlx migrate run

shell-backend:
	docker-compose exec backend bash

shell-frontend:
	docker-compose exec frontend sh

health:
	@echo "Checking all services..."
	@curl -f http://localhost:4000/health || echo "Backend: FAIL"
	@curl -f http://localhost:4003/health || echo "AI Engine: FAIL"
	@curl -f http://localhost:4001 || echo "Frontend: FAIL"