import React from 'react';
import styled from '@emotion/styled';

const StyledButton = styled.button`
  background: var(--color-primary);
  color: var(--color-text);
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);

  &:hover {
    background: var(--color-secondary);
  }
`;

const Button: React.FC<React.ButtonHTMLAttributes<HTMLButtonElement>> = ({
  children,
  ...props
}) => <StyledButton {...props}>{children}</StyledButton>;

export default Button;
