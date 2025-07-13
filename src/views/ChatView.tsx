import React from 'react';
import Assistant from '../components/Assistant';
import Chat from '../components/Chat';
import CodeEditor from '../components/CodeEditor';
import VoiceControls from "../components/Voice";
import Panel from '../components/common/Panel';
import styled from '@emotion/styled';

const Wrapper = styled.div`
  display: flex;
  height: 100%;
`;

const ChatSection = styled(Panel)`
  flex: 1;
  overflow-y: auto;
  margin-right: 0.5rem;
`;

const CodeSection = styled(Panel)`
  width: 40%;
  overflow-y: auto;
`;

const ChatView: React.FC = () => (
  <Wrapper>
    <ChatSection>
      <Assistant />
      <Chat />
      <VoiceControls />
    </ChatSection>
    <CodeSection>
      <CodeEditor />
    </CodeSection>
  </Wrapper>
);

export default ChatView;
