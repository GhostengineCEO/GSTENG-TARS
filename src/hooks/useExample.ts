import { useState } from 'react';

export const useExample = () => {
  const [state, setState] = useState<string>('');
  return { state, setState };
};
