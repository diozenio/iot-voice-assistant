import "./ActionButton.css";

type ActionButtonProps = {
  label: string;
  onClick: () => void;
  variant?: "default" | "primary" | "danger";
  disabled?: boolean;
};

function ActionButton({
  label,
  onClick,
  variant = "default",
  disabled = false,
}: ActionButtonProps) {
  return (
    <button
      className={`action-button action-button-${variant}`}
      onClick={onClick}
      disabled={disabled}
    >
      {label}
    </button>
  );
}

export default ActionButton;
