import styled from '@emotion/styled';
import { motion } from 'framer-motion';

const Container = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
`;

function App() {
  return (
    <Container>
      <motion.h1 initial={{ opacity: 0 }} animate={{ opacity: 1 }}>
        GSTENG - Your Coding Companion
      </motion.h1>
    </Container>
  );
}

export default App;
