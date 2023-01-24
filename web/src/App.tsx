import { useAccount } from 'wagmi';

import { Account, Connect, Login, NetworkSwitcher } from './components';

export function App() {
  const { isConnected } = useAccount();

  return (
    <>
      <h1>Avanguard web3 sign-in demo</h1>

      <Connect />

      {isConnected && (
        <>
          <Account />
          <NetworkSwitcher />
          <Login />
        </>
      )}
    </>
  );
}
