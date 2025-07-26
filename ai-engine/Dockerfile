FROM python:3.12.5-slim AS base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy code
COPY . .

# Create directory for models
RUN mkdir -p /app/models

EXPOSE 4003
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "4003"]
