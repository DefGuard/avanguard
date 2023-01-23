import { useAccount, useEnsName } from 'wagmi';

export const Account = () => {
  const { address } = useAccount();
  const { data: ensName } = useEnsName({ address });

  return (
    <div>
      {ensName ?? address}
      {ensName ? ` (${address})` : null}
    </div>
  );
};
