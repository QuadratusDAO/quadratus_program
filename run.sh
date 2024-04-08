#!/bin/bash

solana transfer 7yvFUSGY5ueMy9K7ihoDuKpbnAbkXsTgEZe7hVooEMN8 10000 --allow-unfunded-recipient

anchor build

anchor test --skip-local-validator