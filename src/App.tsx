import React from 'react';
import styled from '@emotion/styled';
import { Link, Routes, Route } from 'react-router-dom';
import ChatView from './views/ChatView';
import SettingsView from './views/SettingsView';
import ProjectsView from './views/ProjectsView';
import { H1 } from './components/common/Typography';

const Layout = styled.div`
  display: flex;
  height: 100%;
`;

const Sidebar = styled.nav`
  width: 200px;
  background: linear-gradient(180deg, var(--color-background), #111);
  padding: 1rem;
  display: flex;
  flex-direction: column;
`;

const NavLink = styled(Link)`
  color: var(--color-text);
  text-decoration: none;
  margin-bottom: 0.5rem;
  &:hover {
    color: var(--color-primary);
  }
`;

const Main = styled.main`
  flex: 1;
  display: flex;
  padding: 0.5rem;
`;

function App() {
  return (
    <Layout>
      <Sidebar>
        <H1 style={{ marginBottom: '1rem' }}>GSTENG</H1>
        <NavLink to="/">Chat</NavLink>
        <NavLink to="/projects">Projects</NavLink>
        <NavLink to="/settings">Settings</NavLink>
      </Sidebar>
      <Main>
        <Routes>
          <Route path="/" element={<ChatView />} />
          <Route path="/projects" element={<ProjectsView />} />
          <Route path="/settings" element={<SettingsView />} />
        </Routes>
      </Main>
    </Layout>
  );
}

export default App;
