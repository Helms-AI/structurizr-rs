# ADR-012: Design System Architecture

## Status

Accepted

## Context

The platform requires a consistent, scalable design system across:

- Web IDE application
- Marketing website
- Mobile applications
- Documentation
- Admin dashboard

**Requirements:**
- Consistent visual language
- Accessible (WCAG 2.1 AA)
- Dark/light theme support
- Component reusability
- Design-dev handoff efficiency
- Performance optimized

## Decision

We will implement a **design system using TypeScript, React, and Emotion CSS-in-JS**, following atomic design principles with a token-based theming system.

**Key Design:**

1. **Design Tokens**: CSS custom properties for colors, spacing, typography
2. **Component Library**: Atomic components (atoms → molecules → organisms)
3. **Emotion CSS-in-JS**: Type-safe styling with theme support
4. **Storybook**: Component documentation and testing
5. **Figma Integration**: Design tokens synced from Figma

## Alternatives Considered

### Tailwind CSS

**Pros:**
- Utility-first, rapid development
- Small bundle size with purging
- Large community
- Easy responsive design

**Cons:**
- Class name sprawl in JSX
- Less type safety
- Harder to maintain consistent tokens
- Custom component styles verbose

**Why Rejected:** Type-safe theming more important for large codebase.

### Styled Components

**Pros:**
- Popular, well-documented
- Similar to Emotion
- Good TypeScript support

**Cons:**
- Larger bundle size than Emotion
- Slower runtime performance
- Less flexible than Emotion

**Why Rejected:** Emotion offers better performance and flexibility.

### CSS Modules

**Pros:**
- True CSS isolation
- No runtime overhead
- Familiar CSS syntax

**Cons:**
- No dynamic theming
- TypeScript integration complex
- Harder to share tokens

**Why Rejected:** Dynamic theming required for dark/light mode.

### Pre-built UI Libraries (Chakra, MUI)

**Pros:**
- Ready-made components
- Accessibility built-in
- Faster initial development

**Cons:**
- Bundle size overhead
- Opinionated styling
- Customization limits
- Vendor dependency

**Why Rejected:** Need full control for IDE-specific components.

## Consequences

### Positive

- **Type safety**: Full TypeScript integration
- **Theming**: Easy dark/light mode, custom themes
- **Performance**: Efficient CSS generation
- **Developer experience**: Predictable, documented components
- **Consistency**: Single source of truth for design

### Negative

- **Initial investment**: Building component library takes time
- **Learning curve**: Emotion API and design token concepts
- **Bundle size**: CSS-in-JS adds some overhead
- **Build complexity**: Token transformation pipeline

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Initial investment | Start with core components, expand incrementally |
| Learning curve | Comprehensive documentation, examples |
| Bundle size | Code splitting, critical CSS extraction |
| Build complexity | Automated token sync pipeline |

## Implementation

### Design Tokens

```typescript
// tokens/colors.ts
export const colors = {
  // Brand
  brand: {
    primary: '#F26207',
    secondary: '#1C1C1C',
    accent: '#00D9C0',
  },

  // Semantic
  semantic: {
    success: '#22C55E',
    warning: '#F59E0B',
    error: '#EF4444',
    info: '#3B82F6',
  },

  // Neutrals
  neutral: {
    0: '#FFFFFF',
    50: '#F9FAFB',
    100: '#F3F4F6',
    200: '#E5E7EB',
    300: '#D1D5DB',
    400: '#9CA3AF',
    500: '#6B7280',
    600: '#4B5563',
    700: '#374151',
    800: '#1F2937',
    900: '#111827',
    950: '#030712',
  },
} as const;

// tokens/spacing.ts
export const spacing = {
  0: '0',
  1: '4px',
  2: '8px',
  3: '12px',
  4: '16px',
  5: '20px',
  6: '24px',
  8: '32px',
  10: '40px',
  12: '48px',
  16: '64px',
  20: '80px',
  24: '96px',
} as const;

// tokens/typography.ts
export const typography = {
  fontFamily: {
    sans: '"IBM Plex Sans", -apple-system, BlinkMacSystemFont, sans-serif',
    mono: '"IBM Plex Mono", "Fira Code", monospace',
  },
  fontSize: {
    xs: '12px',
    sm: '14px',
    base: '16px',
    lg: '18px',
    xl: '20px',
    '2xl': '24px',
    '3xl': '30px',
    '4xl': '36px',
    '5xl': '48px',
  },
  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
  lineHeight: {
    tight: 1.25,
    normal: 1.5,
    relaxed: 1.75,
  },
} as const;

// tokens/shadows.ts
export const shadows = {
  sm: '0 1px 2px 0 rgb(0 0 0 / 0.05)',
  md: '0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)',
  lg: '0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)',
  xl: '0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)',
} as const;
```

### Theme Definition

```typescript
// theme/index.ts
import { colors, spacing, typography, shadows } from '../tokens';

export interface Theme {
  colors: {
    background: {
      primary: string;
      secondary: string;
      tertiary: string;
      elevated: string;
    };
    text: {
      primary: string;
      secondary: string;
      muted: string;
      inverse: string;
    };
    border: {
      default: string;
      muted: string;
      focus: string;
    };
    interactive: {
      primary: string;
      primaryHover: string;
      primaryActive: string;
      secondary: string;
      secondaryHover: string;
    };
    status: {
      success: string;
      warning: string;
      error: string;
      info: string;
    };
  };
  spacing: typeof spacing;
  typography: typeof typography;
  shadows: typeof shadows;
  borderRadius: {
    sm: string;
    md: string;
    lg: string;
    full: string;
  };
}

export const lightTheme: Theme = {
  colors: {
    background: {
      primary: colors.neutral[0],
      secondary: colors.neutral[50],
      tertiary: colors.neutral[100],
      elevated: colors.neutral[0],
    },
    text: {
      primary: colors.neutral[900],
      secondary: colors.neutral[700],
      muted: colors.neutral[500],
      inverse: colors.neutral[0],
    },
    border: {
      default: colors.neutral[200],
      muted: colors.neutral[100],
      focus: colors.brand.primary,
    },
    interactive: {
      primary: colors.brand.primary,
      primaryHover: '#D85506',
      primaryActive: '#BF4B05',
      secondary: colors.neutral[100],
      secondaryHover: colors.neutral[200],
    },
    status: colors.semantic,
  },
  spacing,
  typography,
  shadows,
  borderRadius: {
    sm: '4px',
    md: '8px',
    lg: '12px',
    full: '9999px',
  },
};

export const darkTheme: Theme = {
  colors: {
    background: {
      primary: colors.neutral[950],
      secondary: colors.neutral[900],
      tertiary: colors.neutral[800],
      elevated: colors.neutral[900],
    },
    text: {
      primary: colors.neutral[50],
      secondary: colors.neutral[300],
      muted: colors.neutral[500],
      inverse: colors.neutral[900],
    },
    border: {
      default: colors.neutral[700],
      muted: colors.neutral[800],
      focus: colors.brand.primary,
    },
    interactive: {
      primary: colors.brand.primary,
      primaryHover: '#FF7A1A',
      primaryActive: '#FF8C33',
      secondary: colors.neutral[800],
      secondaryHover: colors.neutral[700],
    },
    status: colors.semantic,
  },
  spacing,
  typography,
  shadows,
  borderRadius: {
    sm: '4px',
    md: '8px',
    lg: '12px',
    full: '9999px',
  },
};
```

### Theme Provider

```typescript
// theme/ThemeProvider.tsx
import { ThemeProvider as EmotionThemeProvider } from '@emotion/react';
import { createContext, useContext, useState, useEffect } from 'react';
import { lightTheme, darkTheme, Theme } from './index';

type ThemeMode = 'light' | 'dark' | 'system';

interface ThemeContextValue {
  mode: ThemeMode;
  setMode: (mode: ThemeMode) => void;
  theme: Theme;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }
  return context;
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [mode, setMode] = useState<ThemeMode>('system');
  const [resolvedTheme, setResolvedTheme] = useState<Theme>(darkTheme);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    function updateTheme() {
      if (mode === 'system') {
        setResolvedTheme(mediaQuery.matches ? darkTheme : lightTheme);
      } else {
        setResolvedTheme(mode === 'dark' ? darkTheme : lightTheme);
      }
    }

    updateTheme();
    mediaQuery.addEventListener('change', updateTheme);
    return () => mediaQuery.removeEventListener('change', updateTheme);
  }, [mode]);

  return (
    <ThemeContext.Provider value={{ mode, setMode, theme: resolvedTheme }}>
      <EmotionThemeProvider theme={resolvedTheme}>
        {children}
      </EmotionThemeProvider>
    </ThemeContext.Provider>
  );
}
```

### Component Examples

```typescript
// components/Button/Button.tsx
import styled from '@emotion/styled';
import { css } from '@emotion/react';
import { Theme } from '../../theme';

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  fullWidth?: boolean;
  disabled?: boolean;
  loading?: boolean;
  children: React.ReactNode;
  onClick?: () => void;
}

const sizeStyles = {
  sm: css`
    padding: 6px 12px;
    font-size: 14px;
  `,
  md: css`
    padding: 10px 16px;
    font-size: 14px;
  `,
  lg: css`
    padding: 12px 24px;
    font-size: 16px;
  `,
};

const variantStyles = (theme: Theme) => ({
  primary: css`
    background-color: ${theme.colors.interactive.primary};
    color: ${theme.colors.text.inverse};
    border: none;

    &:hover:not(:disabled) {
      background-color: ${theme.colors.interactive.primaryHover};
    }

    &:active:not(:disabled) {
      background-color: ${theme.colors.interactive.primaryActive};
    }
  `,
  secondary: css`
    background-color: ${theme.colors.interactive.secondary};
    color: ${theme.colors.text.primary};
    border: 1px solid ${theme.colors.border.default};

    &:hover:not(:disabled) {
      background-color: ${theme.colors.interactive.secondaryHover};
    }
  `,
  ghost: css`
    background-color: transparent;
    color: ${theme.colors.text.primary};
    border: none;

    &:hover:not(:disabled) {
      background-color: ${theme.colors.interactive.secondary};
    }
  `,
  danger: css`
    background-color: ${theme.colors.status.error};
    color: ${theme.colors.text.inverse};
    border: none;

    &:hover:not(:disabled) {
      background-color: #DC2626;
    }
  `,
});

const StyledButton = styled.button<ButtonProps>`
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: ${({ theme }) => theme.spacing[2]};
  font-family: ${({ theme }) => theme.typography.fontFamily.sans};
  font-weight: ${({ theme }) => theme.typography.fontWeight.medium};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  cursor: pointer;
  transition: all 150ms ease;
  width: ${({ fullWidth }) => (fullWidth ? '100%' : 'auto')};

  ${({ size = 'md' }) => sizeStyles[size]}
  ${({ variant = 'primary', theme }) => variantStyles(theme)[variant]}

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  &:focus-visible {
    outline: 2px solid ${({ theme }) => theme.colors.border.focus};
    outline-offset: 2px;
  }
`;

export function Button({
  variant = 'primary',
  size = 'md',
  fullWidth = false,
  disabled = false,
  loading = false,
  children,
  onClick,
}: ButtonProps) {
  return (
    <StyledButton
      variant={variant}
      size={size}
      fullWidth={fullWidth}
      disabled={disabled || loading}
      onClick={onClick}
    >
      {loading && <Spinner size="sm" />}
      {children}
    </StyledButton>
  );
}

// components/Input/Input.tsx
import styled from '@emotion/styled';
import { forwardRef } from 'react';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
}

const InputWrapper = styled.div`
  display: flex;
  flex-direction: column;
  gap: ${({ theme }) => theme.spacing[1]};
`;

const Label = styled.label`
  font-size: ${({ theme }) => theme.typography.fontSize.sm};
  font-weight: ${({ theme }) => theme.typography.fontWeight.medium};
  color: ${({ theme }) => theme.colors.text.primary};
`;

const StyledInput = styled.input<{ hasError?: boolean }>`
  padding: ${({ theme }) => `${theme.spacing[2]} ${theme.spacing[3]}`};
  font-size: ${({ theme }) => theme.typography.fontSize.base};
  font-family: ${({ theme }) => theme.typography.fontFamily.sans};
  color: ${({ theme }) => theme.colors.text.primary};
  background-color: ${({ theme }) => theme.colors.background.primary};
  border: 1px solid ${({ theme, hasError }) =>
    hasError ? theme.colors.status.error : theme.colors.border.default};
  border-radius: ${({ theme }) => theme.borderRadius.md};
  transition: border-color 150ms ease, box-shadow 150ms ease;

  &:focus {
    outline: none;
    border-color: ${({ theme, hasError }) =>
      hasError ? theme.colors.status.error : theme.colors.border.focus};
    box-shadow: 0 0 0 3px ${({ theme, hasError }) =>
      hasError ? 'rgba(239, 68, 68, 0.2)' : 'rgba(242, 98, 7, 0.2)'};
  }

  &::placeholder {
    color: ${({ theme }) => theme.colors.text.muted};
  }

  &:disabled {
    background-color: ${({ theme }) => theme.colors.background.secondary};
    cursor: not-allowed;
  }
`;

const HelperText = styled.span<{ isError?: boolean }>`
  font-size: ${({ theme }) => theme.typography.fontSize.xs};
  color: ${({ theme, isError }) =>
    isError ? theme.colors.status.error : theme.colors.text.muted};
`;

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, helperText, ...props }, ref) => {
    return (
      <InputWrapper>
        {label && <Label>{label}</Label>}
        <StyledInput ref={ref} hasError={!!error} {...props} />
        {(error || helperText) && (
          <HelperText isError={!!error}>{error || helperText}</HelperText>
        )}
      </InputWrapper>
    );
  }
);
```

### IDE-Specific Components

```typescript
// components/Editor/FileTab.tsx
import styled from '@emotion/styled';
import { FileIcon } from '../icons';

interface FileTabProps {
  filename: string;
  isActive: boolean;
  isDirty: boolean;
  onSelect: () => void;
  onClose: () => void;
}

const TabContainer = styled.div<{ isActive: boolean }>`
  display: flex;
  align-items: center;
  gap: ${({ theme }) => theme.spacing[2]};
  padding: ${({ theme }) => `${theme.spacing[2]} ${theme.spacing[3]}`};
  background-color: ${({ theme, isActive }) =>
    isActive ? theme.colors.background.primary : theme.colors.background.secondary};
  border-bottom: 2px solid ${({ theme, isActive }) =>
    isActive ? theme.colors.interactive.primary : 'transparent'};
  cursor: pointer;
  user-select: none;

  &:hover {
    background-color: ${({ theme, isActive }) =>
      isActive ? theme.colors.background.primary : theme.colors.background.tertiary};
  }
`;

const Filename = styled.span`
  font-size: ${({ theme }) => theme.typography.fontSize.sm};
  color: ${({ theme }) => theme.colors.text.primary};
  font-family: ${({ theme }) => theme.typography.fontFamily.sans};
`;

const DirtyIndicator = styled.span`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: ${({ theme }) => theme.colors.interactive.primary};
`;

const CloseButton = styled.button`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  background: none;
  border: none;
  border-radius: ${({ theme }) => theme.borderRadius.sm};
  color: ${({ theme }) => theme.colors.text.muted};
  cursor: pointer;

  &:hover {
    background-color: ${({ theme }) => theme.colors.background.tertiary};
    color: ${({ theme }) => theme.colors.text.primary};
  }
`;

export function FileTab({ filename, isActive, isDirty, onSelect, onClose }: FileTabProps) {
  return (
    <TabContainer isActive={isActive} onClick={onSelect}>
      <FileIcon filename={filename} size={16} />
      <Filename>{filename}</Filename>
      {isDirty && <DirtyIndicator />}
      <CloseButton
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
      >
        ×
      </CloseButton>
    </TabContainer>
  );
}
```

### Storybook Configuration

```typescript
// .storybook/preview.tsx
import { ThemeProvider } from '../src/theme/ThemeProvider';
import { lightTheme, darkTheme } from '../src/theme';

export const parameters = {
  actions: { argTypesRegex: '^on[A-Z].*' },
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
  backgrounds: {
    default: 'dark',
    values: [
      { name: 'dark', value: darkTheme.colors.background.primary },
      { name: 'light', value: lightTheme.colors.background.primary },
    ],
  },
};

export const decorators = [
  (Story, context) => {
    const theme = context.globals.theme === 'light' ? 'light' : 'dark';
    return (
      <ThemeProvider initialMode={theme}>
        <Story />
      </ThemeProvider>
    );
  },
];

export const globalTypes = {
  theme: {
    name: 'Theme',
    description: 'Global theme for components',
    defaultValue: 'dark',
    toolbar: {
      icon: 'circlehollow',
      items: ['light', 'dark'],
      showName: true,
    },
  },
};
```

## Accessibility Guidelines

| Requirement | Implementation |
|-------------|----------------|
| Color contrast | Minimum 4.5:1 for text, 3:1 for large text |
| Focus indicators | Visible focus rings on all interactive elements |
| Keyboard navigation | All functionality accessible via keyboard |
| Screen readers | Proper ARIA labels and roles |
| Motion | Respect prefers-reduced-motion |

## References

- [Emotion Documentation](https://emotion.sh/docs/introduction)
- [Atomic Design](https://atomicdesign.bradfrost.com/)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [Storybook Documentation](https://storybook.js.org/docs)
