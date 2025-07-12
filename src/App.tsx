import styled from '@emotion/styled';
import { motion } from 'framer-motion';
import Assistant from './components/Assistant';
import Chat from './components/Chat';
import CodeEditor from './components/CodeEditor';
import VoiceControls from './components/Voice';

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
      <Assistant />
      <Chat />
      <CodeEditor />
      <VoiceControls />
    </Container>
  );
}

export default App;
