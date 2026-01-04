import { fireEvent, render, screen } from '@testing-library/react';
import { createRef } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { SearchBar } from './SearchBar';

describe('SearchBar', () => {
  const defaultProps = {
    value: '',
    onChange: vi.fn(),
    onArrowDown: vi.fn(),
    onEscape: vi.fn(),
  };

  it('should render input with placeholder', () => {
    render(<SearchBar {...defaultProps} placeholder="Search..." />);

    const input = screen.getByPlaceholderText('Search...');
    expect(input).toBeDefined();
  });

  it('should display the current value', () => {
    render(<SearchBar {...defaultProps} value="test query" />);

    const input = screen.getByRole('textbox') as HTMLInputElement;
    expect(input.value).toBe('test query');
  });

  it('should call onChange when input value changes', () => {
    const onChange = vi.fn();
    render(<SearchBar {...defaultProps} onChange={onChange} />);

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'new value' } });

    expect(onChange).toHaveBeenCalledWith('new value');
  });

  it('should call onArrowDown when ArrowDown key is pressed', () => {
    const onArrowDown = vi.fn();
    render(<SearchBar {...defaultProps} onArrowDown={onArrowDown} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'ArrowDown' });

    expect(onArrowDown).toHaveBeenCalled();
  });

  it('should call onEscape when Escape key is pressed', () => {
    const onEscape = vi.fn();
    render(<SearchBar {...defaultProps} onEscape={onEscape} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'Escape' });

    expect(onEscape).toHaveBeenCalled();
  });

  it('should be focusable via ref', () => {
    const inputRef = createRef<HTMLInputElement>();
    render(<SearchBar {...defaultProps} inputRef={inputRef} />);

    expect(inputRef.current).toBeDefined();
    expect(inputRef.current?.tagName).toBe('INPUT');
  });

  it('should not call onArrowDown for other keys', () => {
    const onArrowDown = vi.fn();
    render(<SearchBar {...defaultProps} onArrowDown={onArrowDown} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'ArrowUp' });
    fireEvent.keyDown(input, { key: 'Enter' });
    fireEvent.keyDown(input, { key: 'a' });

    expect(onArrowDown).not.toHaveBeenCalled();
  });

  it('should call onArrowLeft when ArrowLeft key is pressed', () => {
    const onArrowLeft = vi.fn();
    render(<SearchBar {...defaultProps} onArrowLeft={onArrowLeft} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'ArrowLeft' });

    expect(onArrowLeft).toHaveBeenCalled();
  });

  it('should call onArrowRight when ArrowRight key is pressed', () => {
    const onArrowRight = vi.fn();
    render(<SearchBar {...defaultProps} onArrowRight={onArrowRight} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'ArrowRight' });

    expect(onArrowRight).toHaveBeenCalled();
  });

  it('should not call onArrowLeft if not provided', () => {
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');
    // Should not throw when ArrowLeft is pressed without handler
    fireEvent.keyDown(input, { key: 'ArrowLeft' });
  });

  it('should show clear button when value is not empty', () => {
    render(<SearchBar {...defaultProps} value="test" />);

    const clearButton = screen.getByRole('button', { name: 'Clear search (Esc)' });
    expect(clearButton).toBeDefined();
  });

  it('should not show clear button when value is empty', () => {
    render(<SearchBar {...defaultProps} value="" />);

    const clearButton = screen.queryByRole('button', { name: 'Clear search (Esc)' });
    expect(clearButton).toBeNull();
  });

  it('should call onChange with empty string when clear button is clicked', () => {
    const onChange = vi.fn();
    render(<SearchBar {...defaultProps} value="test" onChange={onChange} />);

    const clearButton = screen.getByRole('button', { name: 'Clear search (Esc)' });
    fireEvent.click(clearButton);

    expect(onChange).toHaveBeenCalledWith('');
  });

  it('should call onEnter when Enter key is pressed', () => {
    const onEnter = vi.fn();
    render(<SearchBar {...defaultProps} onEnter={onEnter} />);

    const input = screen.getByRole('textbox');
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(onEnter).toHaveBeenCalled();
  });

  it('should not call onEnter if not provided', () => {
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');
    // Should not throw when Enter is pressed without handler
    fireEvent.keyDown(input, { key: 'Enter' });
  });
});
