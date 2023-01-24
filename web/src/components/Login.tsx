import axios from 'axios';
import { useState } from 'react';
import { useCookies } from 'react-cookie';
import ReactJson from 'react-json-view';
import { useAccount, useSignTypedData } from 'wagmi';

import { useApi } from '../shared/hooks/useApi';
import {
  ApiHookAvanguard,
  Claims,
  LoginResponse,
  SignMessageRequest,
  WalletChallenge,
  WalletChallengeRequest,
} from '../shared/types';

const avanguardUrl = process.env.AVANGUARD_BASE_URL;

interface HookProps {
  baseURL?: string;
}

const client = axios.create({
  baseURL:
    avanguardUrl && String(avanguardUrl).length > 0
      ? avanguardUrl
      : 'avanguard',
});
client.defaults.headers.common['Content-Type'] = 'application/json';

const useAvanguardApi = (props?: HookProps): ApiHookAvanguard => {
  if (props) {
    const { baseURL } = props;
    if (baseURL && baseURL.length) {
      client.defaults.baseURL = baseURL;
    }
  }
  const getWalletChallenge = (data: WalletChallengeRequest) =>
    client
      .post<WalletChallenge>(`auth/start`, data)
      .then((response) => response.data);

  const login = (data: SignMessageRequest) =>
    client.post<LoginResponse>(`auth`, data).then((response) => response.data);

  return {
    getWalletChallenge,
    login,
  };
};
const createNonce = (length: number): string => {
  let result = '';
  const characters =
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  const charactersLength = characters.length;
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
};

export const Login = () => {
  const { connector, isConnected, address } = useAccount();
  const { getWalletChallenge, login } = useAvanguardApi();
  const { verifyToken } = useApi();
  const { signTypedDataAsync } = useSignTypedData();
  const [claims, setClaims] = useState<Claims>();
  const [, setCookie] = useCookies(['avanguard_token']);

  const handleLogin = (address: string) => {
    getWalletChallenge({ address })
      .then(async (data) => {
        const message = JSON.parse(data.challenge);
        const types = message.types;
        const domain = message.domain;
        const value = message.message;
        const nonce = createNonce(20);
        const signature = await signTypedDataAsync({ types, domain, value });

        login({ signature, address, nonce }).then((data) => {
          const token = data.token;
          verifyToken({ token, nonce }).then((data) => {
            setClaims(data);
          });
          setCookie('avanguard_token', data.token, { httpOnly: true });
        });
      })
      .catch((err) => console.log(err));
  };

  return (
    <>
      {isConnected && address ? (
        <>
          <h1>Login</h1>
          <button onClick={() => handleLogin(address)}>
            Sign in with {connector?.name}
          </button>
          {claims ? (
            <>
              <h2>Succesfully decoded and validated token</h2>
              <ReactJson src={claims} />
            </>
          ) : null}
        </>
      ) : null}
    </>
  );
};
