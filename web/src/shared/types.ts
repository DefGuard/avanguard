export interface WalletChallengeRequest {
  address: string;
  chainId?: number;
}

export interface LoginResponse {
  token: string;
}

export interface WalletChallenge {
  challenge: string;
}
export interface SignMessageRequest {
  address: string;
  signature: string;
  nonce: string;
}

export interface ValidateTokenRequest {
  nonce: string;
  token: string;
}

export interface ApiHookAvanguard {
  getWalletChallenge: (
    data: WalletChallengeRequest
  ) => Promise<WalletChallenge>;
  login: (data: SignMessageRequest) => Promise<LoginResponse>;
}

export interface Claims {
  iss: string;
  sub: string;
  aud: string[];
  exp: number;
  nonce: string;
}

export interface ApiHook {
  verifyToken: (data: ValidateTokenRequest) => Promise<Claims>;
}
