import React from 'react';
import styled from '@emotion/styled';

const Wrapper = styled.div`
  background: rgba(255, 255, 255, 0.05);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
  padding: 1rem;
  border-radius: 6px;
`;

const Panel: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({ children, ...props }) => (
  <Wrapper {...props}>{children}</Wrapper>
);

export default Panel;
