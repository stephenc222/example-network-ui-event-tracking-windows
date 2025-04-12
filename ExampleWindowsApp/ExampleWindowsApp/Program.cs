using System;
using System.Net.Http;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace ExampleWindowsApp
{
    static class Program
    {
        [STAThread]
        static void Main()
        {
            Application.SetHighDpiMode(HighDpiMode.SystemAware);
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new MainForm());
        }
    }

    public class MainForm : Form
    {
        private readonly Button buttonA;
        private readonly Button buttonB;
        private readonly Button buttonC;
        private readonly TextBox responseTextBox;
        private readonly Label titleLabel;

        public MainForm()
        {
            this.Text = "Example Windows App";
            this.Width = 600;
            this.Height = 400;

            titleLabel = new Label
            {
                Text = "Example Windows App",
                Font = new System.Drawing.Font("Segoe UI", 14, System.Drawing.FontStyle.Bold),
                AutoSize = true,
                Top = 20,
                Left = 20
            };

            buttonA = new Button { Text = "Button A", Name = "ButtonA", Top = 60, Left = 20, Width = 100 };
            buttonB = new Button { Text = "Button B", Name = "ButtonB", Top = 60, Left = 130, Width = 100 };
            buttonC = new Button { Text = "Button C", Name = "ButtonC", Top = 60, Left = 240, Width = 100 };

            responseTextBox = new TextBox
            {
                Multiline = true,
                ScrollBars = ScrollBars.Vertical,
                Top = 100,
                Left = 20,
                Width = 540,
                Height = 200,
                Font = new System.Drawing.Font("Consolas", 10),
                ReadOnly = true
            };

            buttonA.Click += async (_, __) => await FetchTodoAsync(1);
            buttonB.Click += async (_, __) => await FetchTodoAsync(2);
            buttonC.Click += async (_, __) => await FetchTodoAsync(3);

            this.Controls.Add(titleLabel);
            this.Controls.Add(buttonA);
            this.Controls.Add(buttonB);
            this.Controls.Add(buttonC);
            this.Controls.Add(responseTextBox);
        }

        private async Task FetchTodoAsync(int id)
        {
            responseTextBox.Text = $"Loading product {id}...";

            try
            {
                using var client = new HttpClient();
                var response = await client.GetStringAsync($"https://jsonplaceholder.typicode.com/todos/{id}");
                responseTextBox.Text = response;
            }
            catch (Exception ex)
            {
                responseTextBox.Text = $"Error fetching product {id}: {ex.Message}";
            }
        }
    }
}
