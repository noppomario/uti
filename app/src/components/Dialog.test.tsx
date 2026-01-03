/**
 * Tests for Dialog component
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Dialog } from './Dialog';

describe('Dialog', () => {
  it('renders when open is true', () => {
    render(
      <Dialog
        open={true}
        title="Test Title"
        message="Test Message"
        kind="info"
        onClose={() => {}}
      />
    );

    expect(screen.getByText('Test Title')).toBeDefined();
    expect(screen.getByText('Test Message')).toBeDefined();
  });

  it('does not render when open is false', () => {
    render(
      <Dialog
        open={false}
        title="Test Title"
        message="Test Message"
        kind="info"
        onClose={() => {}}
      />
    );

    expect(screen.queryByText('Test Title')).toBeNull();
    expect(screen.queryByText('Test Message')).toBeNull();
  });

  it('displays title and message correctly', () => {
    render(
      <Dialog
        open={true}
        title="Update Available"
        message="Version 1.2.3 is available"
        kind="info"
        onClose={() => {}}
      />
    );

    expect(screen.getByText('Update Available')).toBeDefined();
    expect(screen.getByText('Version 1.2.3 is available')).toBeDefined();
  });

  it('applies info styling for kind="info"', () => {
    render(
      <Dialog
        open={true}
        title="Info Title"
        message="Info Message"
        kind="info"
        onClose={() => {}}
      />
    );

    const title = screen.getByText('Info Title');
    expect(title.className).toContain('text-app-accent-info');
  });

  it('applies error styling for kind="error"', () => {
    render(
      <Dialog
        open={true}
        title="Error Title"
        message="Error Message"
        kind="error"
        onClose={() => {}}
      />
    );

    const title = screen.getByText('Error Title');
    expect(title.className).toContain('text-app-accent-error');
  });

  it('calls onClose when overlay is clicked', () => {
    const handleClose = vi.fn();
    render(<Dialog open={true} title="Test" message="Test" kind="info" onClose={handleClose} />);

    const overlay = screen.getByTestId('dialog-overlay');
    fireEvent.click(overlay);
    expect(handleClose).toHaveBeenCalled();
  });

  it('calls onClose when Escape key is pressed', () => {
    const handleClose = vi.fn();
    render(<Dialog open={true} title="Test" message="Test" kind="info" onClose={handleClose} />);

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(handleClose).toHaveBeenCalled();
  });

  it('calls onClose when OK button is clicked', () => {
    const handleClose = vi.fn();
    render(<Dialog open={true} title="Test" message="Test" kind="info" onClose={handleClose} />);

    const okButton = screen.getByRole('button', { name: 'OK' });
    fireEvent.click(okButton);
    expect(handleClose).toHaveBeenCalled();
  });

  it('does not call onClose when dialog content is clicked', () => {
    const handleClose = vi.fn();
    render(<Dialog open={true} title="Test" message="Test" kind="info" onClose={handleClose} />);

    const dialogContent = screen.getByTestId('dialog-content');
    fireEvent.click(dialogContent);
    expect(handleClose).not.toHaveBeenCalled();
  });

  it('preserves whitespace in message', () => {
    render(
      <Dialog
        open={true}
        title="Test"
        message={'Line 1\nLine 2\nLine 3'}
        kind="info"
        onClose={() => {}}
      />
    );

    const message = screen.getByText(/Line 1/);
    expect(message.className).toContain('whitespace-pre-wrap');
  });
});
