import React from 'react';
import styled from '@emotion/styled';

const SpinnerWrapper = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
`;

const Block = styled.div`
  width: 8px;
  height: 8px;
  margin: 2px;
  background: var(--color-primary);
  animation: spin 1s infinite ease-in-out;

  @keyframes spin {
    0%, 80%, 100% {
      transform: scale(0);
    }
    40% {
      transform: scale(1);
    }
  }
`;

const LoadingSpinner: React.FC = () => (
  <SpinnerWrapper>
    {Array.from({ length: 3 }).map((_, i) => (
      <Block key={i} style={{ animationDelay: `${i * 0.2}s` }} />
    ))}
  </SpinnerWrapper>
);

export default LoadingSpinner;
