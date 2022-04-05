import React from 'react';
import logo from './logo.svg';
import './App.css';
import { AppBar, Avatar, Badge, Box, Card, CardContent, Grid, IconButton, Paper, Toolbar, Typography } from '@mui/material';
import MenuIcon from '@mui/icons-material/Menu';
import { createTheme, styled } from '@mui/material/styles';
import { alpha, ThemeProvider } from '@mui/system';
import { pink } from '@mui/material/colors';
import SearchIcon from '@mui/icons-material/Search';
import AccountCircle from '@mui/icons-material/AccountCircle';
import MailIcon from '@mui/icons-material/Mail';
import NotificationsIcon from '@mui/icons-material/Notifications';
import MoreIcon from '@mui/icons-material/MoreVert';
import { BrowserRouter, Routes, Route } from 'react-router-dom'

const theme = createTheme(
  {
    palette: {
      primary: pink,
      secondary: pink,
    },
    typography: {
      fontFamily: 'sans-serif',
    }
  }
);

let StyledCard = styled(Card)({
  transition: 'all 0.2s ease-in-out',
  '&:hover': {
    backgroundColor: alpha(theme.palette.primary.main, 0.1),
  },
});

function Home() {
  return (
    <Box sx={{ flexGrow: 1, overflow: 'hidden', px: 3 }}>
      <StyledCard
        variant="outlined"
        sx={{
          my: 2,
          mx: 'auto',
          p: 2,
          maxWidth: 600,
        }}
      >
        <Grid container wrap="nowrap" spacing={2}>
          <Grid item>
            <img style={{width: 100, height: 100}} src={"http://www.databaseguides.com/wp-content/uploads/2009/06/postgresql-logo.png"} alt="logo" />
          </Grid>
          <Grid item xs zeroMinWidth>
            <Typography noWrap variant='h3'>Postgres</Typography>
            <Typography noWrap >Lorem ipsum etc</Typography>
          </Grid>
        </Grid>
      </StyledCard>
    </Box>
  );
}

function App() {
  return (
    <div className="App">
      <BrowserRouter>
        <ThemeProvider theme={theme} >
          <AppBar position="static">
            <Toolbar>
              <IconButton
                size="large"
                edge="start"
                color="inherit"
                aria-label="open drawer"
                sx={{ mr: 2 }}
              >
                <MenuIcon />
              </IconButton>
              <Typography
                variant="h6"
                noWrap
                component="div"
                sx={{ display: { xs: 'none', sm: 'block' } }}
              >
                MUI
              </Typography>
              <Box sx={{ flexGrow: 1 }} />
              <Box sx={{ display: { xs: 'none', md: 'flex' } }}>
                <IconButton size="large" aria-label="show 4 new mails" color="inherit">
                  <MailIcon />
                </IconButton>
                <IconButton
                  size="large"
                  aria-label="show 17 new notifications"
                  color="inherit"
                >
                  <NotificationsIcon />
                </IconButton>
                <IconButton
                  size="large"
                  edge="end"
                  aria-label="account of current user"
                  aria-haspopup="true"
                  color="inherit"
                >
                  <AccountCircle />
                </IconButton>
              </Box>
              <Box sx={{ display: { xs: 'flex', md: 'none' } }}>
                <IconButton
                  size="large"
                  aria-label="show more"
                  aria-haspopup="true"
                  color="inherit"
                >
                  <MoreIcon />
                </IconButton>
              </Box>
            </Toolbar>
          </AppBar>
          <Routes>
            <Route index element={<Home />} />
          </Routes>
        </ThemeProvider>
      </BrowserRouter>
    </div>
  );
}

export default App;
