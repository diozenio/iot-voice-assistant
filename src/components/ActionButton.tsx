import "./ActionButton.css";

type ActionButtonProps = {
  label: string;
  onClick: () => void;
};

function ActionButton({ label, onClick }: ActionButtonProps) {
  return (
    <button className="action-button" onClick={onClick}>
      {label}
    </button>
  );
}

export default ActionButton;
