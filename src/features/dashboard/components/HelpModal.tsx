import Modal from "../../../components/Modal";
import "./HelpModal.css";

type HelpModalProps = {
  onClose: () => void;
};

const COMMANDS = [
  { action: "Abrir o navegador", example: "“Abrir navegador”" },
  { action: "Pesquisar no Google", example: "“Pesquisar receita de bolo”" },
  {
    action: "Pesquisar no YouTube",
    example: "“Pesquisar no YouTube música relaxante”",
  },
  { action: "Abrir um site", example: "“Abrir github.com”" },
  { action: "Abrir o explorador de arquivos", example: "“Abrir explorador”" },
  {
    action: "Abrir uma pasta (downloads, documentos, área de trabalho, imagens)",
    example: "“Abrir downloads”",
  },
  { action: "Abrir a calculadora", example: "“Abrir calculadora”" },
  {
    action: "Responder",
    example: "Qualquer outra pergunta, como “Quanto é 12 vezes 8?”",
  },
];

function HelpModal({ onClose }: HelpModalProps) {
  return (
    <Modal title="Ajuda — comandos de voz" onClose={onClose}>
      <ol className="help-steps">
        <li>Aperte o botão do ESP32 (o LED acende) e fale o comando.</li>
        <li>Aperte o botão de novo para parar de gravar.</li>
        <li>A IA entende o pedido, executa a ação e responde.</li>
      </ol>

      <table className="help-table">
        <thead>
          <tr>
            <th>Ação</th>
            <th>Exemplo de fala</th>
          </tr>
        </thead>
        <tbody>
          {COMMANDS.map((command) => (
            <tr key={command.action}>
              <td>{command.action}</td>
              <td>{command.example}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <p className="help-note">
        Gravações com menos de 0,5 s são ignoradas. Fale com clareza e perto do
        microfone.
      </p>
    </Modal>
  );
}

export default HelpModal;
